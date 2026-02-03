use async_trait::async_trait;
use database::{
    ApplicationTransientStatus, DownloadableMetadata, borrow_db_checked, borrow_db_mut_checked,
};
use download_manager::depot_manager::DepotManager;
use download_manager::download_manager_frontend::{DownloadManagerSignal, DownloadStatus};
use download_manager::downloadable::Downloadable;
use download_manager::error::ApplicationDownloadError;
use download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use download_manager::util::progress_object::{ProgressHandle, ProgressObject, ProgressType};
use droplet_rs::manifest::{ChunkData, Manifest};
use log::{debug, error, info, warn};
use remote::auth::generate_authorization_header;
use remote::error::RemoteAccessError;
use remote::requests::generate_url;
use remote::utils::DROP_CLIENT_ASYNC;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::AppHandle;
use tokio::sync::mpsc::Sender;
use utils::{app_emit, lock, send};

use crate::downloads::utils::get_disk_available;
use crate::library::{on_game_complete, push_game_update, set_partially_installed};
use crate::state::GameStatusManager;

use super::download_logic::download_game_chunk;
use super::drop_data::DropData;

static RETRY_COUNT: usize = 3;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInformation {
    file_list: HashMap<String, String>,
    manifests: HashMap<String, Manifest>,
    install_size: u64,
    download_size: u64,
}

pub struct GameDownloadAgent {
    pub metadata: DownloadableMetadata,
    pub control_flag: DownloadThreadControl,
    pub dl_info: Mutex<Option<DownloadInformation>>,
    pub download_progress: Arc<ProgressObject>,
    pub disk_progress: Arc<ProgressObject>,
    depot_manager: Arc<DepotManager>,
    sender: Sender<DownloadManagerSignal>,
    pub dropdata: DropData,
    status: Mutex<DownloadStatus>,
}

impl Debug for GameDownloadAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameDownloadAgent").finish()
    }
}

impl GameDownloadAgent {
    pub async fn new_from_index(
        metadata: DownloadableMetadata,
        target_download_dir: usize,
        sender: Sender<DownloadManagerSignal>,
        depot_manager: Arc<DepotManager>,
    ) -> Result<Self, ApplicationDownloadError> {
        let base_dir = {
            let db_lock = borrow_db_checked();

            db_lock.applications.install_dirs[target_download_dir].clone()
        };

        Self::new(metadata, base_dir, sender, depot_manager).await
    }
    pub async fn new(
        metadata: DownloadableMetadata,
        base_dir: PathBuf,
        sender: Sender<DownloadManagerSignal>,
        depot_manager: Arc<DepotManager>,
    ) -> Result<Self, ApplicationDownloadError> {
        // Don't run by default
        let control_flag = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);

        let base_dir_path = Path::new(&base_dir);
        info!("base dir {}", base_dir_path.display());
        let data_base_dir_path = base_dir_path.join(metadata.id.clone());
        info!("data dir path {}", data_base_dir_path.display());

        let stored_manifest = DropData::generate(
            metadata.id.clone(),
            metadata.version.clone(),
            metadata.target_platform,
            data_base_dir_path.clone(),
        );

        let result = Self {
            metadata,
            control_flag,
            dl_info: Mutex::new(None),
            download_progress: Arc::new(ProgressObject::new(
                0,
                0,
                sender.clone(),
                ProgressType::Download,
            )),
            disk_progress: Arc::new(ProgressObject::new(
                0,
                0,
                sender.clone(),
                ProgressType::Disk,
            )),
            sender,
            dropdata: stored_manifest,
            status: Mutex::new(DownloadStatus::Queued),
            depot_manager,
        };

        result.ensure_manifest_exists().await?;

        let required_space = lock!(result.dl_info).as_ref().unwrap().install_size;

        let available_space = get_disk_available(data_base_dir_path)? as u64;

        if required_space > available_space {
            return Err(ApplicationDownloadError::DiskFull(
                required_space,
                available_space,
            ));
        }

        Ok(result)
    }

    // Blocking
    pub fn setup_download(&self, app_handle: &AppHandle) -> Result<(), ApplicationDownloadError> {
        let mut db_lock = borrow_db_mut_checked();
        let status = ApplicationTransientStatus::Downloading {
            version_id: self.metadata.version.clone(),
        };
        db_lock
            .applications
            .transient_statuses
            .insert(self.metadata(), status.clone());
        // Don't use GameStatusManager because this game isn't installed
        push_game_update(app_handle, &self.metadata().id, None, (None, Some(status)));

        if !self.check_manifest_exists() {
            return Err(ApplicationDownloadError::NotInitialized);
        }

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub async fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        self.setup_download(app_handle)?;
        let timer = Instant::now();

        info!("beginning download for {}...", self.metadata().id);

        let res = self
            .run()
            .await
            .map_err(ApplicationDownloadError::Communication);

        debug!(
            "{} took {}ms to download",
            self.metadata.id,
            timer.elapsed().as_millis()
        );
        res
    }

    pub fn check_manifest_exists(&self) -> bool {
        lock!(self.dl_info).is_some()
    }

    pub async fn ensure_manifest_exists(&self) -> Result<(), ApplicationDownloadError> {
        if lock!(self.dl_info).is_some() {
            return Ok(());
        }

        self.download_manifest().await
    }

    async fn download_manifest(&self) -> Result<(), ApplicationDownloadError> {
        let client = DROP_CLIENT_ASYNC.clone();
        let url = generate_url(
            &["/api/v1/client/game/manifest"],
            &[
                ("id", &self.metadata.id),
                ("version", &self.metadata.version),
            ],
        )
        .map_err(ApplicationDownloadError::Communication)?;

        let response = client
            .get(url)
            .header("Authorization", generate_authorization_header())
            .send()
            .await
            .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

        if response.status() != 200 {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::ManifestDownloadFailed(
                    response.status(),
                    response.text().await.unwrap(),
                ),
            ));
        }

        let manifest_download: DownloadInformation = response
            .json()
            .await
            .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

        if let Ok(mut manifest) = self.dl_info.lock() {
            *manifest = Some(manifest_download);
            return Ok(());
        }

        Err(ApplicationDownloadError::Lock)
    }

    // Sets up progress for download writes
    fn setup_progress(&self) {
        let dl_info = lock!(self.dl_info);
        let dl_info = dl_info.as_ref().unwrap();

        let total_chunks = dl_info
            .manifests
            .iter()
            .map(|v| v.1.chunks.len())
            .sum::<usize>();

        self.download_progress
            .set_max(dl_info.download_size.try_into().unwrap());
        self.download_progress
            .set_size(total_chunks);
        self.download_progress.reset();

        self.disk_progress.set_max(dl_info.install_size.try_into().unwrap());
        self.disk_progress
            .set_size(total_chunks);
        self.disk_progress.reset();
    }

    async fn run(&self) -> Result<bool, RemoteAccessError> {
        self.depot_manager.sync_depots().await?;
        info!("synced depots");
        self.setup_progress();
        info!("setup progress objects");
        let manifests_chunks: Vec<(String, HashMap<String, ChunkData>, [u8; 16])> = {
            let dl_info = lock!(self.dl_info);
            dl_info
                .as_ref()
                .unwrap()
                .manifests
                .iter()
                .map(|v| (v.0.clone(), v.1.chunks.clone(), v.1.key))
                .collect()
        };
        let file_list = {
            let dl_info = lock!(self.dl_info);
            dl_info.as_ref().unwrap().file_list.clone()
        };
        let mut completed_chunks = {
            let completed_chunks = lock!(self.dropdata.contexts);
            completed_chunks.clone()
        };
        let chunk_len = manifests_chunks.iter().map(|v| v.1.len()).sum::<usize>();
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        let (sender, recv) = crossbeam_channel::bounded(16);

        // SAFETY: I pinky-promise
        // (the scope keeps these in scope)
        let unsafe_self: &'static GameDownloadAgent = unsafe { mem::transmute(self) };
        let file_list: &'static HashMap<String, String> = unsafe { mem::transmute(&file_list) };

        let local_completed_chunks = completed_chunks.clone();

        let download_join_handle = tauri::async_runtime::spawn_blocking(move || {
            let thread_pool = rayon::ThreadPoolBuilder::new()
                .num_threads(max_download_threads)
                .build()
                .unwrap();
            thread_pool.scope(move |s| {
                let mut index = 0;
                for (version_id, chunks, key) in manifests_chunks.into_iter() {
                    let version_id = &version_id;
                    for (chunk_id, chunk_data) in chunks.into_iter() {
                        let local_sender = sender.clone();
                        let download_progress_handle = ProgressHandle::new(
                            unsafe_self.download_progress.get(index),
                            unsafe_self.download_progress.clone(),
                        );
                        let disk_progress_handle = ProgressHandle::new(
                            unsafe_self.disk_progress.get(index),
                            unsafe_self.disk_progress.clone(),
                        );
                        index += 1;

                        let chunk_length = chunk_data.files.iter().map(|v| v.length).sum();

                        if *local_completed_chunks.get(&chunk_id).unwrap_or(&false) {
                            download_progress_handle.skip(chunk_length);
                            continue;
                        }

                        let sender = unsafe_self.sender.clone();
                        let (depot, permit) = match unsafe_self
                            .depot_manager
                            .next_depot(&unsafe_self.metadata.id, &unsafe_self.metadata.version)
                        {
                            Ok(v) => v,
                            Err(err) => {
                                tauri::async_runtime::spawn(async move {
                                    send!(
                                        sender,
                                        DownloadManagerSignal::Error(
                                            ApplicationDownloadError::Communication(err)
                                        )
                                    );
                                });
                                return;
                            }
                        };

                        let local_version_id = version_id.clone();
                        s.spawn(move |_| {
                            for i in 0..RETRY_COUNT {
                                let base_path = unsafe_self.dropdata.base_path.clone();
                                match download_game_chunk(
                                    &unsafe_self.metadata.id,
                                    &local_version_id,
                                    &chunk_id,
                                    &depot,
                                    &key,
                                    &chunk_data,
                                    file_list,
                                    base_path,
                                    &unsafe_self.control_flag,
                                    &download_progress_handle,
                                    &disk_progress_handle,
                                ) {
                                    Ok(true) => {
                                        local_sender.send(chunk_id.clone()).unwrap();
                                        drop(permit); // Take ownership
                                        return;
                                    }
                                    Ok(false) => return,
                                    Err(e) => {
                                        warn!("got error for chunk id {}: {e:?}", chunk_id);

                                        let retry = true; /*matches!(
                                        &e,
                                        ApplicationDownloadError::Communication(_)
                                        | ApplicationDownloadError::Checksum
                                        | ApplicationDownloadError::Lock
                                        | ApplicationDownloadError::IoError(_)
                                        );*/

                                        if i == RETRY_COUNT - 1 || !retry {
                                            warn!("retry logic failed, not re-attempting.");
                                            tauri::async_runtime::spawn(async move {
                                                send!(sender, DownloadManagerSignal::Error(e));
                                            });
                                            return;
                                        }
                                    }
                                }
                            }
                        });
                    }
                }

                drop(sender);
            });
        });

        let mut outputs = Vec::new();
        while let Ok(chunk_id) = recv.recv() {
            outputs.push(chunk_id);
        }

        download_join_handle
            .await
            .expect("failed to complete download");

        for completed_chunk in outputs {
            completed_chunks.insert(completed_chunk, true);
        }

        let drop_data_chunks = completed_chunks
            .iter()
            .map(|v| (v.0.to_string(), *v.1))
            .collect::<Vec<(String, bool)>>();

        self.dropdata.set_contexts(&drop_data_chunks);
        self.dropdata.write();

        info!("completed {} chunks", drop_data_chunks.len());

        // If there are any contexts left which are false
        if completed_chunks.len() != chunk_len {
            info!(
                "download agent for {} exited without completing ({}/{})",
                self.metadata.id.clone(),
                completed_chunks.len(),
                chunk_len,
            );
            return Ok(false);
        }
        Ok(true)
    }

    #[allow(dead_code)]
    fn setup_validate(&self, app_handle: &AppHandle) {
        self.setup_progress();

        self.control_flag.set(DownloadThreadControlFlag::Go);

        let status = ApplicationTransientStatus::Validating {
            version_id: self.metadata.version.clone(),
        };

        let mut db_lock = borrow_db_mut_checked();
        db_lock
            .applications
            .transient_statuses
            .insert(self.metadata(), status.clone());
        push_game_update(app_handle, &self.metadata().id, None, (None, Some(status)));
    }

    pub fn validate(&self, _app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        /*
        self.setup_validate(app_handle);

        let buckets = lock!(self.buckets);
        let contexts: Vec<DropValidateContext> = buckets
            .clone()
            .into_iter()
            .flat_map(|e| -> Vec<DropValidateContext> { e.into() })
            .collect();
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        info!("{} validation contexts", contexts.len());
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_download_threads)
            .build()
            .unwrap_or_else(|_| {
                panic!("failed to build thread pool with {max_download_threads} threads")
            });

        let invalid_chunks = Arc::new(boxcar::Vec::new());
        pool.scope(|scope| {
            for (index, context) in contexts.iter().enumerate() {
                let current_progress = self.progress.get(index);
                let progress_handle = ProgressHandle::new(current_progress, self.progress.clone());
                let invalid_chunks_scoped = invalid_chunks.clone();
                let sender = self.sender.clone();

                scope.spawn(move |_| {
                    match validate_game_chunk(context, &self.control_flag, progress_handle) {
                        Ok(true) => {}
                        Ok(false) => {
                            invalid_chunks_scoped.push(context.checksum.clone());
                        }
                        Err(e) => {
                            error!("{e}");
                            send!(sender, DownloadManagerSignal::Error(e));
                        }
                    }
                });
            }
        });

        // If there are any contexts left which are false
        if !invalid_chunks.is_empty() {
            info!("validation of game id {} failed", self.id);

            for context in invalid_chunks.iter() {
                self.dropdata.set_context(context.1.clone(), false);
            }

            self.dropdata.write();

            return Ok(false);
        }
         */

        Ok(true)
    }

    pub fn cancel(&self, app_handle: &AppHandle) {
        // See docs on usage
        set_partially_installed(
            &self.metadata(),
            self.dropdata.base_path.display().to_string(),
            Some(app_handle),
        );

        self.dropdata.write();
    }
}

#[async_trait]
impl Downloadable for GameDownloadAgent {
    async fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *lock!(self.status) = DownloadStatus::Downloading;
        self.download(app_handle).await
    }

    fn validate(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *lock!(self.status) = DownloadStatus::Validating;
        self.validate(app_handle)
    }

    fn dl_progress(&self) -> &Arc<ProgressObject> {
        &self.download_progress
    }

    fn disk_progress(&self) -> &Arc<ProgressObject> {
        &self.disk_progress
    }

    fn control_flag(&self) -> DownloadThreadControl {
        self.control_flag.clone()
    }

    fn metadata(&self) -> DownloadableMetadata {
        self.metadata.clone()
    }

    fn on_queued(&self, app_handle: &tauri::AppHandle) {
        *self.status.lock().unwrap() = DownloadStatus::Queued;
        let mut db_lock = borrow_db_mut_checked();
        let status = ApplicationTransientStatus::Queued {
            version_id: self.metadata.version.clone(),
        };
        db_lock
            .applications
            .transient_statuses
            .insert(self.metadata(), status.clone());
        push_game_update(app_handle, &self.metadata.id, None, (None, Some(status)));
    }

    fn on_error(&self, app_handle: &tauri::AppHandle, error: &ApplicationDownloadError) {
        *lock!(self.status) = DownloadStatus::Error;
        app_emit!(app_handle, "download_error", error.to_string());

        error!("error while managing download: {error:?}");

        let mut handle = borrow_db_mut_checked();
        handle
            .applications
            .transient_statuses
            .remove(&self.metadata());

        push_game_update(
            app_handle,
            &self.metadata.id,
            None,
            GameStatusManager::fetch_state(&self.metadata.id, &handle),
        );
    }

    async fn on_complete(&self, app_handle: &tauri::AppHandle) {
        match on_game_complete(
            &self.metadata(),
            self.dropdata.base_path.to_string_lossy().to_string(),
            app_handle,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("could not mark game as complete: {e}");
                send!(
                    self.sender,
                    DownloadManagerSignal::Error(ApplicationDownloadError::DownloadError(e))
                );
            }
        }
    }

    fn on_cancelled(&self, app_handle: &tauri::AppHandle) {
        self.cancel(app_handle);
    }

    fn status(&self) -> DownloadStatus {
        lock!(self.status).clone()
    }
}
