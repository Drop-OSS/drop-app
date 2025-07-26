use crate::DB;
use crate::auth::generate_authorization_header;
use crate::database::db::{DatabaseImpls, borrow_db_checked, borrow_db_mut_checked};
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
use crate::games::downloads::validate::game_validate_logic;
use crate::games::library::{on_game_complete, on_game_incomplete, push_game_update};
use crate::remote::requests::make_request;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs::{OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

#[cfg(target_os = "linux")]
use rustix::fs::{FallocateFlags, fallocate};

use super::download_logic::download_game_chunk;
use super::drop_data::DropData;

// This is cursed but necessary
// See the message where it is used
unsafe fn extend_lifetime<'b, R>(r: &'b R) -> &'static R {
    unsafe { std::mem::transmute::<&'b R, &'static R>(r) }
}

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    pub control_flag: DownloadThreadControl,
    contexts: Mutex<Vec<DropDownloadContext>>,
    context_map: Mutex<HashMap<String, bool>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    pub stored_manifest: DropData,
    status: Mutex<DownloadStatus>,
}

impl GameDownloadAgent {
    pub async fn new_from_index(
        id: String,
        version: String,
        target_download_dir: usize,
        sender: Sender<DownloadManagerSignal>,
    ) -> Self {
        let db_lock = borrow_db_checked().await;
        let base_dir = db_lock.applications.install_dirs[target_download_dir].clone();
        drop(db_lock);

        Self::new(id, version, base_dir, sender)
    }
    pub fn new(
        id: String,
        version: String,
        base_dir: PathBuf,
        sender: Sender<DownloadManagerSignal>,
    ) -> Self {
        // Don't run by default
        let control_flag = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);

        let base_dir_path = Path::new(&base_dir);
        let data_base_dir_path = base_dir_path.join(id.clone());

        let stored_manifest =
            DropData::generate(id.clone(), version.clone(), data_base_dir_path.clone());

        Self {
            id,
            version,
            control_flag,
            manifest: Mutex::new(None),
            contexts: Mutex::new(Vec::new()),
            context_map: Mutex::new(HashMap::new()),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            stored_manifest,
            status: Mutex::new(DownloadStatus::Queued),
        }
    }

    // Blocking
    pub async fn setup_download(&self) -> Result<(), ApplicationDownloadError> {
        self.ensure_manifest_exists().await?;

        self.ensure_contexts()?;

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub async fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        debug!("starting download");
        self.setup_download().await?;
        self.set_progress_object_params();
        let timer = Instant::now();
        push_game_update(
            app_handle,
            &self.metadata().id,
            None,
            (
                None,
                Some(ApplicationTransientStatus::Downloading {
                    version_name: self.version.clone(),
                }),
            ),
        );
        let res = self
            .run()
            .await
            .map_err(|_| ApplicationDownloadError::DownloadError);

        debug!(
            "{} took {}ms to download",
            self.id,
            timer.elapsed().as_millis()
        );
        res
    }

    pub async fn ensure_manifest_exists(&self) -> Result<(), ApplicationDownloadError> {
        if self.manifest.lock().unwrap().is_some() {
            return Ok(());
        }

        self.download_manifest().await
    }

    async fn download_manifest(&self) -> Result<(), ApplicationDownloadError> {
        let header = generate_authorization_header().await;
        let client = reqwest::Client::new();
        let response = make_request(
            &client,
            &["/api/v1/client/game/manifest"],
            &[("id", &self.id), ("version", &self.version)],
            async |f| f.header("Authorization", header),
        )
        .await
        .map_err(ApplicationDownloadError::Communication)?
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

        let manifest_download: DropManifest = response.json().await.unwrap();

        if let Ok(mut manifest) = self.manifest.lock() {
            *manifest = Some(manifest_download);
            return Ok(());
        }

        Err(ApplicationDownloadError::Lock)
    }

    fn set_progress_object_params(&self) {
        // Avoid re-setting it
        if self.progress.get_max() != 0 {
            return;
        }

        let contexts = self.contexts.lock().unwrap();

        let length = contexts.len();

        let chunk_count = contexts.iter().map(|chunk| chunk.length).sum();

        self.progress.set_max(chunk_count);
        self.progress.set_size(length);
        self.progress.set_time_now();
    }

    pub fn ensure_contexts(&self) -> Result<(), ApplicationDownloadError> {
        if self.contexts.lock().unwrap().is_empty() {
            self.generate_contexts()?;
        }

        self.context_map
            .lock()
            .unwrap()
            .extend(self.stored_manifest.get_contexts());

        Ok(())
    }

    pub fn generate_contexts(&self) -> Result<(), ApplicationDownloadError> {
        let manifest = self.manifest.lock().unwrap().clone().unwrap();
        let game_id = self.id.clone();

        let mut contexts = Vec::new();
        let base_path = Path::new(&self.stored_manifest.base_path);
        create_dir_all(base_path).unwrap();

        for (raw_path, chunk) in manifest {
            let path = base_path.join(Path::new(&raw_path));

            let container = path.parent().unwrap();
            create_dir_all(container).unwrap();

            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .create(true)
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
            if running_offset > 0 {
                let _ = fallocate(file, FallocateFlags::empty(), 0, running_offset);
            }
        }
        let existing_contexts = self.stored_manifest.get_completed_contexts();
        self.stored_manifest.set_contexts(
            &contexts
                .iter()
                .map(|x| (x.checksum.clone(), existing_contexts.contains(&x.checksum)))
                .collect::<Vec<(String, bool)>>(),
        );

        *self.contexts.lock().unwrap() = contexts;

        Ok(())
    }

    pub async fn run(&self) -> Result<bool, ()> {
        let max_download_threads = borrow_db_checked().await.settings.max_download_threads;

        debug!(
            "downloading game: {} with {} threads",
            self.id, max_download_threads
        );

        let base_url = DB
            .fetch_base_url()
            .await
            .join("/api/v1/client/chunk")
            .unwrap();
        let client = reqwest::Client::new();
        let client_ref = unsafe { extend_lifetime(&client) };

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(max_download_threads)
            .thread_name("drop-download-thread")
            .enable_io()
            .enable_time()
            .build()
            .map_err(|e| {
                warn!("failed to create download scheduler: {e}");
                ()
            })?;

        let (tx, mut rx) = mpsc::channel(32);

        // Scope this for safety
        {
            let contexts = self.contexts.lock().unwrap();
            debug!("{contexts:#?}");

            let context_map = self.context_map.lock().unwrap();
            for (index, context) in contexts.iter().enumerate() {
                let progress = self.progress.get(index);
                let progress_handle = ProgressHandle::new(progress, self.progress.clone());

                // If we've done this one already, skip it
                if Some(&true) == context_map.get(&context.checksum) {
                    progress_handle.skip(context.length);
                    continue;
                }

                let sender = self.sender.clone();

                let local_tx = tx.clone();
                /*
                This lifetime extensions are necessary, because this loop acts like a scope
                but Rust doesn't know that.
                */
                let context = unsafe { extend_lifetime(context) };
                let self_static = unsafe { extend_lifetime(self) };
                let mut base_url = base_url.clone();
                rt.spawn(async move {
                    {
                        let mut query = base_url.query_pairs_mut();
                        let query_params = [
                            ("id", &context.game_id),
                            ("version", &context.version),
                            ("name", &context.file_name),
                            ("chunk", &context.index.to_string()),
                        ];
                        for (param, val) in query_params {
                            query.append_pair(param.as_ref(), val.as_ref());
                        }
                    }

                    let request = client_ref.get(base_url);

                    match download_game_chunk(
                        context,
                        &self_static.control_flag,
                        progress_handle,
                        request,
                    )
                    .await
                    {
                        Ok(true) => {
                            local_tx.send(context.checksum.clone()).await.unwrap();
                        }
                        Ok(false) => {}
                        Err(e) => {
                            error!("{e}");
                            sender.send(DownloadManagerSignal::Error(e)).unwrap();
                        }
                    }
                });
            }
        }

        let mut newly_completed = Vec::new();
        while let Some(completed_checksum) = rx.recv().await {
            newly_completed.push(completed_checksum);
        }

        // 'return' from the download
        let mut context_map_lock = self.context_map.lock().unwrap();
        for item in newly_completed.iter() {
            context_map_lock.insert(item.clone(), true);
        }
        let completed_lock_len = context_map_lock.values().filter(|x| **x).count();

        let contexts = self.contexts.lock().unwrap();
        let contexts = contexts
            .iter()
            .map(|x| {
                (
                    x.checksum.clone(),
                    context_map_lock.get(&x.checksum).cloned().unwrap_or(false),
                )
            })
            .collect::<Vec<(String, bool)>>();

        drop(context_map_lock);

        self.stored_manifest.set_contexts(&contexts);
        self.stored_manifest.write();

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
}

#[async_trait::async_trait]
impl Downloadable for GameDownloadAgent {
    fn metadata(&self) -> DownloadableMetadata {
        DownloadableMetadata {
            id: self.id.clone(),
            version: Some(self.version.clone()),
            download_type: DownloadType::Game,
        }
    }

    async fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        debug!("starting download from downloadable trait");
        *self.status.lock().unwrap() = DownloadStatus::Downloading;
        self.download(app_handle).await
    }

    async fn progress(&self) -> Arc<ProgressObject> {
        self.progress.clone()
    }

    async fn control_flag(&self) -> DownloadThreadControl {
        self.control_flag.clone()
    }

    async fn on_initialised(&self, _app_handle: &tauri::AppHandle) {
        *self.status.lock().unwrap() = DownloadStatus::Queued;
    }

    async fn on_error(&self, app_handle: &tauri::AppHandle, error: &ApplicationDownloadError) {
        *self.status.lock().unwrap() = DownloadStatus::Error;
        app_handle
            .emit("download_error", error.to_string())
            .unwrap();

        error!("error while managing download: {error}");

        let mut handle = borrow_db_mut_checked().await;
        handle
            .applications
            .transient_statuses
            .remove(&self.metadata());
    }

    async fn on_complete(&self, app_handle: &tauri::AppHandle) {
        on_game_complete(
            &self.metadata(),
            self.stored_manifest.base_path.to_string_lossy().to_string(),
            app_handle,
        )
        .await
        .unwrap();
    }

    async fn on_incomplete(&self, app_handle: &tauri::AppHandle) {
        on_game_incomplete(
            &self.metadata(),
            self.stored_manifest.base_path.to_string_lossy().to_string(),
            app_handle,
        )
        .await
        .unwrap();
    }

    async fn on_cancelled(&self, _app_handle: &tauri::AppHandle) {}

    async fn status(&self) -> DownloadStatus {
        self.status.lock().unwrap().clone()
    }

    async fn validate(&self) -> Result<bool, ApplicationDownloadError> {
        *self.status.lock().unwrap() = DownloadStatus::Validating;
        let contexts = self.contexts.lock().unwrap().clone();
        game_validate_logic(
            &self.stored_manifest,
            contexts,
            self.progress.clone(),
            self.sender.clone(),
            &self.control_flag,
        )
        .await
    }
}
