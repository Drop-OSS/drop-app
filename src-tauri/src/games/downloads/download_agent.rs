use crate::auth::generate_authorization_header;
use crate::database::db::{borrow_db_checked, borrow_db_mut_checked};
use crate::database::models::data::{
    ApplicationTransientStatus, DownloadType, DownloadableMetadata,
};
use crate::download_manager::download_manager_frontend::{DownloadManagerSignal, DownloadStatus};
use crate::download_manager::downloadable::Downloadable;
use crate::download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use crate::download_manager::util::progress_object::{ProgressHandle, ProgressObject};
use crate::error::application_download_error::ApplicationDownloadError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::games::downloads::validate::validate_game_chunk;
use crate::games::library::{on_game_complete, push_game_update, set_partially_installed};
use crate::games::state::GameStatusManager;
use crate::process::utils::get_disk_available;
use crate::remote::requests::make_request;
use crate::remote::utils::DROP_CLIENT_SYNC;
use log::{debug, error, info, warn};
use rayon::ThreadPoolBuilder;
use std::collections::HashMap;
use std::fs::{OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "linux")]
use rustix::fs::{FallocateFlags, fallocate};

use super::download_logic::download_game_chunk;
use super::drop_data::DropData;

static RETRY_COUNT: usize = 3;

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    pub control_flag: DownloadThreadControl,
    contexts: Mutex<Vec<DropDownloadContext>>,
    context_map: Mutex<HashMap<String, bool>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    pub dropdata: DropData,
    status: Mutex<DownloadStatus>,
}

impl GameDownloadAgent {
    pub fn new_from_index(
        id: String,
        version: String,
        target_download_dir: usize,
        sender: Sender<DownloadManagerSignal>,
    ) -> Result<Self, ApplicationDownloadError> {
        let db_lock = borrow_db_checked();
        let base_dir = db_lock.applications.install_dirs[target_download_dir].clone();
        drop(db_lock);

        Self::new(id, version, base_dir, sender)
    }
    pub fn new(
        id: String,
        version: String,
        base_dir: PathBuf,
        sender: Sender<DownloadManagerSignal>,
    ) -> Result<Self, ApplicationDownloadError> {
        // Don't run by default
        let control_flag = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);

        let base_dir_path = Path::new(&base_dir);
        let data_base_dir_path = base_dir_path.join(id.clone());

        let stored_manifest =
            DropData::generate(id.clone(), version.clone(), data_base_dir_path.clone());

        let result = Self {
            id,
            version,
            control_flag,
            manifest: Mutex::new(None),
            contexts: Mutex::new(Vec::new()),
            context_map: Mutex::new(HashMap::new()),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            dropdata: stored_manifest,
            status: Mutex::new(DownloadStatus::Queued),
        };

        result.ensure_manifest_exists()?;

        let required_space = result
            .manifest
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .values()
            .map(|e| e.lengths.iter().sum::<usize>())
            .sum::<usize>()
            as u64;

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
        self.ensure_manifest_exists()?;

        self.ensure_contexts()?;

        self.control_flag.set(DownloadThreadControlFlag::Go);

        let mut db_lock = borrow_db_mut_checked();
        db_lock.applications.transient_statuses.insert(
            self.metadata(),
            ApplicationTransientStatus::Downloading {
                version_name: self.version.clone(),
            },
        );
        push_game_update(
            app_handle,
            &self.metadata().id,
            None,
            GameStatusManager::fetch_state(&self.metadata().id, &db_lock),
        );

        Ok(())
    }

    // Blocking
    pub fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        self.setup_download(app_handle)?;
        let timer = Instant::now();

        info!("beginning download for {}...", self.metadata().id);

        let res = self
            .run()
            .map_err(|()| ApplicationDownloadError::DownloadError);

        debug!(
            "{} took {}ms to download",
            self.id,
            timer.elapsed().as_millis()
        );
        res
    }

    pub fn ensure_manifest_exists(&self) -> Result<(), ApplicationDownloadError> {
        if self.manifest.lock().unwrap().is_some() {
            return Ok(());
        }

        self.download_manifest()
    }

    fn download_manifest(&self) -> Result<(), ApplicationDownloadError> {
        let header = generate_authorization_header();
        let client = DROP_CLIENT_SYNC.clone();
        let response = make_request(
            &client,
            &["/api/v1/client/game/manifest"],
            &[("id", &self.id), ("version", &self.version)],
            |f| f.header("Authorization", header),
        )
        .map_err(ApplicationDownloadError::Communication)?
        .send()
        .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

        if response.status() != 200 {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::ManifestDownloadFailed(
                    response.status(),
                    response.text().unwrap(),
                ),
            ));
        }

        let manifest_download: DropManifest = response.json().unwrap();

        if let Ok(mut manifest) = self.manifest.lock() {
            *manifest = Some(manifest_download);
            return Ok(());
        }

        Err(ApplicationDownloadError::Lock)
    }

    // Sets it up for both download and validate
    fn setup_progress(&self) {
        let contexts = self.contexts.lock().unwrap();

        let length = contexts.len();

        let chunk_count = contexts.iter().map(|chunk| chunk.length).sum();

        self.progress.set_max(chunk_count);
        self.progress.set_size(length);
        self.progress.reset();
    }

    pub fn ensure_contexts(&self) -> Result<(), ApplicationDownloadError> {
        if self.contexts.lock().unwrap().is_empty() {
            self.generate_contexts()?;
        }

        *self.context_map.lock().unwrap() = self.dropdata.get_contexts();

        Ok(())
    }

    pub fn generate_contexts(&self) -> Result<(), ApplicationDownloadError> {
        let manifest = self.manifest.lock().unwrap().clone().unwrap();
        let game_id = self.id.clone();

        let mut contexts = Vec::new();
        let base_path = Path::new(&self.dropdata.base_path);
        create_dir_all(base_path).unwrap();

        for (raw_path, chunk) in manifest {
            let path = base_path.join(Path::new(&raw_path));

            let container = path.parent().unwrap();
            create_dir_all(container).unwrap();

            let already_exists = path.exists();
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path.clone())
                .unwrap();
            let mut running_offset = 0;

            for (index, length) in chunk.lengths.iter().enumerate() {
                contexts.push(DropDownloadContext {
                    file_name: raw_path.to_string(),
                    version: chunk.version_name.to_string(),
                    offset: running_offset,
                    index,
                    game_id: game_id.to_string(),
                    path: path.clone(),
                    checksum: chunk.checksums[index].clone(),
                    length: *length,
                    permissions: chunk.permissions,
                });
                running_offset += *length as u64;
            }

            #[cfg(target_os = "linux")]
            if running_offset > 0 && !already_exists {
                let _ = fallocate(file, FallocateFlags::empty(), 0, running_offset);
            }
        }
        let existing_contexts = self.dropdata.get_completed_contexts();
        self.dropdata.set_contexts(
            &contexts
                .iter()
                .map(|x| (x.checksum.clone(), existing_contexts.contains(&x.checksum)))
                .collect::<Vec<(String, bool)>>(),
        );

        *self.contexts.lock().unwrap() = contexts;

        Ok(())
    }

    fn run(&self) -> Result<bool, ()> {
        self.setup_progress();
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        debug!(
            "downloading game: {} with {} threads",
            self.id, max_download_threads
        );
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_download_threads)
            .build()
            .unwrap();

        let completed_contexts = Arc::new(boxcar::Vec::new());
        let completed_indexes_loop_arc = completed_contexts.clone();

        let contexts = self.contexts.lock().unwrap();
        pool.scope(|scope| {
            let client = &DROP_CLIENT_SYNC.clone();
            let context_map = self.context_map.lock().unwrap();
            for (index, context) in contexts.iter().enumerate() {
                let client = client.clone();
                let completed_indexes = completed_indexes_loop_arc.clone();

                let progress = self.progress.get(index);
                let progress_handle = ProgressHandle::new(progress, self.progress.clone());

                // If we've done this one already, skip it
                // Note to future DecDuck, DropData gets loaded into context_map
                if let Some(v) = context_map.get(&context.checksum)
                    && *v
                {
                    progress_handle.skip(context.length);
                    continue;
                }

                let sender = self.sender.clone();

                let request = match make_request(
                    &client,
                    &["/api/v1/client/chunk"],
                    &[
                        ("id", &context.game_id),
                        ("version", &context.version),
                        ("name", &context.file_name),
                        ("chunk", &context.index.to_string()),
                    ],
                    |r| r,
                ) {
                    Ok(request) => request,
                    Err(e) => {
                        sender
                            .send(DownloadManagerSignal::Error(
                                ApplicationDownloadError::Communication(e),
                            ))
                            .unwrap();
                        continue;
                    }
                };

                scope.spawn(move |_| {
                    // 3 attempts
                    for i in 0..RETRY_COUNT {
                        let loop_progress_handle = progress_handle.clone();
                        match download_game_chunk(
                            context,
                            &self.control_flag,
                            loop_progress_handle,
                            request.try_clone().unwrap(),
                        ) {
                            Ok(true) => {
                                completed_indexes.push(context.checksum.clone());
                                return;
                            }
                            Ok(false) => return,
                            Err(e) => {
                                warn!("game download agent error: {e}");

                                let retry = match &e {
                                    ApplicationDownloadError::Communication(
                                        _remote_access_error,
                                    ) => true,
                                    ApplicationDownloadError::Checksum => true,
                                    ApplicationDownloadError::Lock => true,
                                    ApplicationDownloadError::IoError(_error_kind) => false,
                                    ApplicationDownloadError::DownloadError => false,
                                    ApplicationDownloadError::DiskFull(_, _) => false,
                                };

                                if i == RETRY_COUNT - 1 || !retry {
                                    warn!("retry logic failed, not re-attempting.");
                                    sender.send(DownloadManagerSignal::Error(e)).unwrap();
                                    return;
                                }
                            }
                        }
                    }
                });
            }
        });

        let newly_completed = completed_contexts.clone();

        let completed_lock_len = {
            let mut context_map_lock = self.context_map.lock().unwrap();
            for (_, item) in newly_completed.iter() {
                context_map_lock.insert(item.clone(), true);
            }

            context_map_lock.values().filter(|x| **x).count()
        };
        let context_map_lock = self.context_map.lock().unwrap();
        let contexts = contexts
            .iter()
            .map(|x| {
                (
                    x.checksum.clone(),
                    context_map_lock.get(&x.checksum).copied().unwrap_or(false),
                )
            })
            .collect::<Vec<(String, bool)>>();
        drop(context_map_lock);

        self.dropdata.set_contexts(&contexts);
        self.dropdata.write();

        // If there are any contexts left which are false
        if !contexts.iter().all(|x| x.1) {
            info!(
                "download agent for {} exited without completing ({}/{})",
                self.id.clone(),
                completed_lock_len,
                contexts.len(),
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn setup_validate(&self, app_handle: &AppHandle) {
        self.setup_progress();

        self.control_flag.set(DownloadThreadControlFlag::Go);

        let mut db_lock = borrow_db_mut_checked();
        db_lock.applications.transient_statuses.insert(
            self.metadata(),
            ApplicationTransientStatus::Validating {
                version_name: self.version.clone(),
            },
        );
        push_game_update(
            app_handle,
            &self.metadata().id,
            None,
            GameStatusManager::fetch_state(&self.metadata().id, &db_lock),
        );
    }

    pub fn validate(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        self.setup_validate(app_handle);

        let contexts = self.contexts.lock().unwrap();
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        debug!(
            "validating game: {} with {} threads",
            self.dropdata.game_id, max_download_threads
        );
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_download_threads)
            .build()
            .unwrap();

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
                            sender.send(DownloadManagerSignal::Error(e)).unwrap();
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

        Ok(true)
    }

    pub fn cancel(&self, app_handle: &AppHandle) {
        // See docs on usage
        set_partially_installed(
            &self.metadata(),
            self.dropdata.base_path.to_str().unwrap().to_string(),
            Some(app_handle),
        );

        self.dropdata.write();
    }
}

impl Downloadable for GameDownloadAgent {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *self.status.lock().unwrap() = DownloadStatus::Downloading;
        self.download(app_handle)
    }

    fn validate(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *self.status.lock().unwrap() = DownloadStatus::Validating;
        self.validate(app_handle)
    }

    fn progress(&self) -> Arc<ProgressObject> {
        self.progress.clone()
    }

    fn control_flag(&self) -> DownloadThreadControl {
        self.control_flag.clone()
    }

    fn metadata(&self) -> DownloadableMetadata {
        DownloadableMetadata {
            id: self.id.clone(),
            version: Some(self.version.clone()),
            download_type: DownloadType::Game,
        }
    }

    fn on_initialised(&self, _app_handle: &tauri::AppHandle) {
        *self.status.lock().unwrap() = DownloadStatus::Queued;
    }

    fn on_error(&self, app_handle: &tauri::AppHandle, error: &ApplicationDownloadError) {
        *self.status.lock().unwrap() = DownloadStatus::Error;
        app_handle
            .emit("download_error", error.to_string())
            .unwrap();

        error!("error while managing download: {error}");

        let mut handle = borrow_db_mut_checked();
        handle
            .applications
            .transient_statuses
            .remove(&self.metadata());
    }

    fn on_complete(&self, app_handle: &tauri::AppHandle) {
        on_game_complete(
            &self.metadata(),
            self.dropdata.base_path.to_string_lossy().to_string(),
            app_handle,
        )
        .unwrap();
    }

    fn on_cancelled(&self, app_handle: &tauri::AppHandle) {
        self.cancel(app_handle);
        /*
           on_game_incomplete(
               &self.metadata(),
               self.dropdata.base_path.to_string_lossy().to_string(),
               app_handle,
           )
           .unwrap();
        */
    }

    fn status(&self) -> DownloadStatus {
        self.status.lock().unwrap().clone()
    }
}
