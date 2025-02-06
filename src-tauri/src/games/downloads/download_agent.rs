use crate::auth::generate_authorization_header;
use crate::database::db::{
    borrow_db_checked, set_game_status, ApplicationTransientStatus, DatabaseImpls,
    GameDownloadStatus,
};
use crate::download_manager::download_manager::{DownloadManagerSignal, DownloadStatus};
use crate::download_manager::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use crate::download_manager::downloadable::Downloadable;
use crate::download_manager::downloadable_metadata::{DownloadType, DownloadableMetadata};
use crate::download_manager::progress_object::{ProgressHandle, ProgressObject};
use crate::error::application_download_error::ApplicationDownloadError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::games::library::{on_game_complete, push_game_update, GameUpdateEvent};
use crate::remote::requests::make_request;
use crate::DB;
use log::{debug, error, info};
use rayon::ThreadPoolBuilder;
use slice_deque::SliceDeque;
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use urlencoding::encode;

#[cfg(target_os = "linux")]
use rustix::fs::{fallocate, FallocateFlags};

use super::download_logic::download_game_chunk;
use super::stored_manifest::StoredManifest;

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    pub control_flag: DownloadThreadControl,
    contexts: Mutex<Vec<DropDownloadContext>>,
    completed_contexts: Mutex<SliceDeque<usize>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    pub stored_manifest: StoredManifest,
    status: Mutex<DownloadStatus>,
}

impl GameDownloadAgent {
    pub fn new(
        id: String,
        version: String,
        target_download_dir: usize,
        sender: Sender<DownloadManagerSignal>,
    ) -> Self {
        // Don't run by default
        let control_flag = DownloadThreadControl::new(DownloadThreadControlFlag::Stop);

        let db_lock = borrow_db_checked();
        let base_dir = db_lock.applications.install_dirs[target_download_dir].clone();
        drop(db_lock);

        let base_dir_path = Path::new(&base_dir);
        let data_base_dir_path = base_dir_path.join(id.clone());

        let stored_manifest =
            StoredManifest::generate(id.clone(), version.clone(), data_base_dir_path.clone());

        Self {
            id,
            version,
            control_flag,
            manifest: Mutex::new(None),
            contexts: Mutex::new(Vec::new()),
            completed_contexts: Mutex::new(SliceDeque::new()),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            stored_manifest,
            status: Mutex::new(DownloadStatus::Queued),
        }
    }

    // Blocking
    pub fn setup_download(&self) -> Result<(), ApplicationDownloadError> {
        self.ensure_manifest_exists()?;

        self.ensure_contexts()?;

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        self.setup_download()?;
        self.set_progress_object_params();
        let timer = Instant::now();
        push_game_update(
            app_handle,
            &self.metadata().id,
            (
                None,
                Some(ApplicationTransientStatus::Downloading {
                    version_name: self.version.clone(),
                }),
            ),
        );
        let res = self
            .run()
            .map_err(|_| ApplicationDownloadError::DownloadError);

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
        let client = reqwest::blocking::Client::new();
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
        if !self.contexts.lock().unwrap().is_empty() {
            return Ok(());
        }

        self.generate_contexts()?;
        Ok(())
    }

    pub fn generate_contexts(&self) -> Result<(), ApplicationDownloadError> {
        let manifest = self.manifest.lock().unwrap().clone().unwrap();
        let game_id = self.id.clone();

        let mut contexts = Vec::new();
        let base_path = Path::new(&self.stored_manifest.base_path);
        create_dir_all(base_path).unwrap();

        {
            let mut completed_contexts_lock = self.completed_contexts.lock().unwrap();
            completed_contexts_lock.clear();
            completed_contexts_lock
                .extend_from_slice(&self.stored_manifest.get_completed_contexts());
        }

        for (raw_path, chunk) in manifest {
            let path = base_path.join(Path::new(&raw_path));

            let container = path.parent().unwrap();
            create_dir_all(container).unwrap();

            let file = File::create(path.clone()).unwrap();
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
        *self.contexts.lock().unwrap() = contexts;

        Ok(())
    }

    // TODO: Change return value on Err
    pub fn run(&self) -> Result<bool, ()> {
        let max_download_threads = borrow_db_checked().settings.max_download_threads;

        debug!(
            "downloading game: {} with {} threads",
            self.id, max_download_threads
        );
        let pool = ThreadPoolBuilder::new()
            .num_threads(max_download_threads)
            .build()
            .unwrap();

        let completed_indexes = Arc::new(boxcar::Vec::new());
        let completed_indexes_loop_arc = completed_indexes.clone();

        let contexts = self.contexts.lock().unwrap();
        pool.scope(|scope| {
            let client = &reqwest::blocking::Client::new();
            for (index, context) in contexts.iter().enumerate() {
                let client = client.clone();
                let completed_indexes = completed_indexes_loop_arc.clone();

                let progress = self.progress.get(index);
                let progress_handle = ProgressHandle::new(progress, self.progress.clone());

                // If we've done this one already, skip it
                if self.completed_contexts.lock().unwrap().contains(&index) {
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
                    |r| r.header("Authorization", generate_authorization_header()),
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
                    match download_game_chunk(context, &self.control_flag, progress_handle, request)
                    {
                        Ok(res) => {
                            if res {
                                completed_indexes.push(index);
                            }
                        }
                        Err(e) => {
                            error!("{}", e);
                            sender.send(DownloadManagerSignal::Error(e)).unwrap();
                        }
                    }
                });
            }
        });

        let newly_completed = completed_indexes.to_owned();

        let completed_lock_len = {
            let mut completed_contexts_lock = self.completed_contexts.lock().unwrap();
            for (_, item) in newly_completed.iter() {
                completed_contexts_lock.push_front(*item);
            }

            completed_contexts_lock.len()
        };

        // If we're not out of contexts, we're not done, so we don't fire completed
        if completed_lock_len != contexts.len() {
            info!(
                "download agent for {} exited without completing ({}/{})",
                self.id.clone(),
                completed_lock_len,
                contexts.len(),
            );
            self.stored_manifest
                .set_completed_contexts(self.completed_contexts.lock().unwrap().as_slice());
            self.stored_manifest.write();
            return Ok(false);
        }

        // We've completed
        self.sender
            .send(DownloadManagerSignal::Completed(self.metadata()))
            .unwrap();

        Ok(true)
    }
}

impl Downloadable for GameDownloadAgent {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        *self.status.lock().unwrap() = DownloadStatus::Downloading;
        self.download(app_handle)
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

        error!("error while managing download: {}", error);

        set_game_status(app_handle, self.metadata(), |db_handle, meta| {
            db_handle.applications.transient_statuses.remove(meta);
        });
    }

    fn on_complete(&self, app_handle: &tauri::AppHandle) {
        on_game_complete(
            &self.metadata(),
            self.stored_manifest.base_path.to_string_lossy().to_string(),
            app_handle,
        )
        .unwrap();
    }

    // TODO: fix this function. It doesn't restart the download properly, nor does it reset the state properly
    fn on_incomplete(&self, app_handle: &tauri::AppHandle) {
        let meta = self.metadata();
        *self.status.lock().unwrap() = DownloadStatus::Queued;
        app_handle
            .emit(
                &format!("update_game/{}", meta.id),
                GameUpdateEvent {
                    game_id: meta.id.clone(),
                    status: (Some(GameDownloadStatus::Remote {}), None),
                },
            )
            .unwrap();
    }

    fn on_cancelled(&self, _app_handle: &tauri::AppHandle) {}

    fn status(&self) -> DownloadStatus {
        self.status.lock().unwrap().clone()
    }
}
