use crate::auth::generate_authorization_header;
use crate::db::{set_game_status, GameDownloadStatus, ApplicationTransientStatus, DatabaseImpls};
use crate::download_manager::application_download_error::ApplicationDownloadError;
use crate::download_manager::download_manager::{DownloadManagerSignal, DownloadStatus};
use crate::download_manager::download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag};
use crate::download_manager::downloadable::Downloadable;
use crate::download_manager::downloadable_metadata::{DownloadType, DownloadableMetadata};
use crate::download_manager::progress_object::{ProgressHandle, ProgressObject};
use crate::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::library::{on_game_complete, push_game_update};
use crate::remote::RemoteAccessError;
use crate::DB;
use log::{debug, error, info};
use rayon::ThreadPoolBuilder;
use tauri::{AppHandle, Emitter};
use std::collections::VecDeque;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Instant;
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
    completed_contexts: Mutex<VecDeque<usize>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    pub stored_manifest: StoredManifest,
    status: Mutex<DownloadStatus>
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

        let db_lock = DB.borrow_data().unwrap();
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
            completed_contexts: Mutex::new(VecDeque::new()),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            stored_manifest,
            status: Mutex::new(DownloadStatus::Queued),
        }
    }

    // Blocking
    pub fn setup_download(&self) -> Result<(), ApplicationDownloadError> {
        self.ensure_manifest_exists()?;
        info!("Ensured manifest exists");

        self.ensure_contexts()?;
        info!("Ensured contexts exists");

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        info!("Setting up download");
        self.setup_download()?;
        info!("Setting progress object params");
        self.set_progress_object_params();
        info!("Running");
        let timer = Instant::now();
        push_game_update(app_handle, &self.metadata(), (None, Some(ApplicationTransientStatus::Downloading { version_name: self.version.clone() })));
        let res = self.run().map_err(|_| ApplicationDownloadError::DownloadError);

        info!(
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
        let base_url = DB.fetch_base_url();
        let manifest_url = base_url
            .join(
                format!(
                    "/api/v1/client/metadata/manifest?id={}&version={}",
                    self.id,
                    encode(&self.version)
                )
                .as_str(),
            )
            .unwrap();

        let header = generate_authorization_header();
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(manifest_url.to_string())
            .header("Authorization", header)
            .send()
            .unwrap();

        if response.status() != 200 {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::ManifestDownloadFailed(
                    response.status(),
                    response.text().unwrap(),
                ),
            ));
        }

        let manifest_download = response.json::<DropManifest>().unwrap();

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

        debug!("Setting ProgressObject max to {}", chunk_count);
        self.progress.set_max(chunk_count);
        debug!("Setting ProgressObject size to {}", length);
        self.progress.set_size(length);
        debug!("Setting ProgressObject time to now");
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
                .extend(self.stored_manifest.get_completed_contexts());
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

    pub fn run(&self) -> Result<bool, ()> {
        info!("downloading game: {}", self.id);
        const DOWNLOAD_MAX_THREADS: usize = 1;

        let pool = ThreadPoolBuilder::new()
            .num_threads(DOWNLOAD_MAX_THREADS)
            .build()
            .unwrap();

        let completed_indexes = Arc::new(boxcar::Vec::new());
        let completed_indexes_loop_arc = completed_indexes.clone();

        pool.scope(|scope| {
            for (index, context) in self.contexts.lock().unwrap().iter().enumerate() {
                info!("Running index {}", index);
                let completed_indexes = completed_indexes_loop_arc.clone();

                let progress = self.progress.get(index); // Clone arcs
                let progress_handle = ProgressHandle::new(progress, self.progress.clone());
                // If we've done this one already, skip it
                if self.completed_contexts.lock().unwrap().contains(&index) {
                    progress_handle.add(context.length);
                    continue;
                }

                let context = context.clone();
                let control_flag = self.control_flag.clone(); // Clone arcs

                let sender = self.sender.clone();

                scope.spawn(move |_| {
                    match download_game_chunk(context.clone(), control_flag, progress_handle) {
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
            for (item, _) in newly_completed.iter() {
                completed_contexts_lock.push_front(item);
            }

            completed_contexts_lock.len()
        };

        // If we're not out of contexts, we're not done, so we don't fire completed
        if completed_lock_len != self.contexts.lock().unwrap().len() {
            info!("da for {} exited without completing", self.id.clone());
            self.stored_manifest
                .set_completed_contexts(&self.completed_contexts.lock().unwrap().clone().into());
            info!("Setting completed contexts");
            self.stored_manifest.write();
            info!("Wrote completed contexts");
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
        return;
    }

    fn on_error(&self, app_handle: &tauri::AppHandle, error: ApplicationDownloadError) {
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
        on_game_complete(&self.metadata(), self.stored_manifest.base_path.to_string_lossy().to_string(), app_handle).unwrap();
    }

    fn on_incomplete(&self, app_handle: &tauri::AppHandle) {
        *self.status.lock().unwrap() = DownloadStatus::Queued;
        return;
    }

    fn on_cancelled(&self, app_handle: &tauri::AppHandle) {
        return;
    }

    fn on_uninstall(&self, app_handle: &tauri::AppHandle) {
        let mut db_handle = DB.borrow_data_mut().unwrap();
        let metadata = self.metadata();
        db_handle
            .applications
            .transient_statuses
            .entry(metadata.clone())
            .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});
        
        push_game_update(
            app_handle,
            &metadata,
            (None, Some(ApplicationTransientStatus::Uninstalling {})),
        );

        let previous_state = db_handle.applications.game_statuses.get(&metadata.id).cloned();
        if previous_state.is_none() {
            info!("uninstall job doesn't have previous state, failing silently");
            return;
        }
        let previous_state = previous_state.unwrap();
        if let Some((version_name, install_dir)) = match previous_state {
            GameDownloadStatus::Installed {
                version_name,
                install_dir,
            } => Some((version_name, install_dir)),
            GameDownloadStatus::SetupRequired {
                version_name,
                install_dir,
            } => Some((version_name, install_dir)),
            _ => None,
        } {
            db_handle
                .applications
                .transient_statuses
                .entry(metadata.clone())
                .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});
            drop(db_handle);

            let sender = self.sender.clone();
            let app_handle = app_handle.clone();
            spawn(move || match remove_dir_all(install_dir) {
                Err(e) => {
                    sender
                        .send(DownloadManagerSignal::Error(ApplicationDownloadError::IoError(
                            e.kind(),
                        )))
                        .unwrap();
                }
                Ok(_) => {
                    let mut db_handle = DB.borrow_data_mut().unwrap();
                    db_handle.applications.transient_statuses.remove(&metadata);
                    db_handle
                        .applications
                        .game_statuses
                        .entry(metadata.id.clone())
                        .and_modify(|e| *e = GameDownloadStatus::Remote {});
                    drop(db_handle);
                    DB.save().unwrap();

                    info!("uninstalled game id {}", metadata.id);

                    push_game_update(&app_handle, &metadata, (Some(GameDownloadStatus::Remote {}), None));
                }
            });
        }
    }
    
    fn status(&self) -> DownloadStatus {
        self.status.lock().unwrap().clone()
    }
}