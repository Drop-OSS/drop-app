use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::downloads::progress_object::ProgressHandle;
use crate::remote::RemoteAccessError;
use crate::DB;
use core::time;
use log::{debug, error, info};
use rayon::ThreadPoolBuilder;
use serde::ser::{Error, SerializeMap};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, File};
use std::io;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use urlencoding::encode;

#[cfg(target_os = "linux")]
use rustix::fs::{fallocate, FallocateFlags};

use super::download_logic::download_game_chunk;
use super::download_manager::DownloadManagerSignal;
use super::download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag};
use super::progress_object::ProgressObject;
use super::stored_manifest::StoredManifest;

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    pub control_flag: DownloadThreadControl,
    contexts: Vec<DropDownloadContext>,
    completed_contexts: Mutex<Vec<usize>>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    pub stored_manifest: StoredManifest,
}

#[derive(Debug)]
pub enum GameDownloadError {
    Communication(RemoteAccessError),
    Checksum,
    Setup(SetupError),
    Lock,
    IoError(io::Error),
    DownloadError,
}

#[derive(Debug)]
pub enum SetupError {
    Context,
}

impl Display for GameDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDownloadError::Communication(error) => write!(f, "{}", error),
            GameDownloadError::Setup(error) => write!(f, "An error occurred while setting up the download: {}", error),
            GameDownloadError::Lock => write!(f, "Failed to acquire lock. Something has gone very wrong internally. Please restart the application"),
            GameDownloadError::Checksum => write!(f, "Checksum failed to validate for download"),
            GameDownloadError::IoError(error) => write!(f, "{}", error),
            GameDownloadError::DownloadError => write!(f, "Download failed. See Download Manager status for specific error"),
        }
    }
}

impl Display for SetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupError::Context => write!(f, "Failed to generate contexts for download"),
        }
    }
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
        let base_dir = db_lock.games.install_dirs[target_download_dir].clone();
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
            contexts: Vec::new(),
            completed_contexts: Mutex::new(Vec::new()),
            progress: Arc::new(ProgressObject::new(0, 0, sender.clone())),
            sender,
            stored_manifest,
        }
    }

    // Blocking
    pub fn setup_download(&mut self) -> Result<(), GameDownloadError> {
        self.ensure_manifest_exists()?;
        info!("Ensured manifest exists");

        self.ensure_contexts()?;
        info!("Ensured contexts exists");

        self.control_flag.set(DownloadThreadControlFlag::Go);

        Ok(())
    }

    // Blocking
    pub fn download(&mut self) -> Result<(), GameDownloadError> {
        self.setup_download()?;
        self.set_progress_object_params();
        let timer = Instant::now();
        self.run().map_err(|_| GameDownloadError::DownloadError)?;

        info!(
            "{} took {}ms to download",
            self.id,
            timer.elapsed().as_millis()
        );
        Ok(())
    }

    pub fn ensure_manifest_exists(&self) -> Result<(), GameDownloadError> {
        if self.manifest.lock().unwrap().is_some() {
            return Ok(());
        }

        self.download_manifest()
    }

    fn download_manifest(&self) -> Result<(), GameDownloadError> {
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
            return Err(GameDownloadError::Communication(
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

        Err(GameDownloadError::Lock)
    }

    fn set_progress_object_params(&self) {
        // Avoid re-setting it
        if self.progress.get_max() != 0 {
            return;
        }

        let length = self.contexts.len();

        let chunk_count = self.contexts.iter().map(|chunk| chunk.length).sum();

        debug!("Setting ProgressObject max to {}", chunk_count);
        self.progress.set_max(chunk_count);
        debug!("Setting ProgressObject size to {}", length);
        self.progress.set_size(length);
        debug!("Setting ProgressObject time to now");
        self.progress.set_time_now();
    }

    pub fn ensure_contexts(&mut self) -> Result<(), GameDownloadError> {
        if !self.contexts.is_empty() {
            return Ok(());
        }

        self.generate_contexts()?;
        Ok(())
    }

    pub fn generate_contexts(&mut self) -> Result<(), GameDownloadError> {
        let manifest = self.manifest.lock().unwrap().clone().unwrap();
        let game_id = self.id.clone();

        let mut contexts = Vec::new();
        let base_path = Path::new(&self.stored_manifest.base_path);
        create_dir_all(base_path).unwrap();

        *self.completed_contexts.lock().unwrap() = self.stored_manifest.get_completed_contexts();

        info!(
            "Completed contexts: {:?}",
            *self.completed_contexts.lock().unwrap()
        );

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
        self.contexts = contexts;

        Ok(())
    }

    pub fn run(&self) -> Result<(), ()> {
        info!("downloading game: {}", self.id);
        const DOWNLOAD_MAX_THREADS: usize = 1;

        let pool = ThreadPoolBuilder::new()
            .num_threads(DOWNLOAD_MAX_THREADS)
            .build()
            .unwrap();

        let completed_indexes = Arc::new(Mutex::new(Vec::new()));
        let completed_indexes_loop_arc = completed_indexes.clone();

        pool.scope(move |scope| {
            let completed_lock = self.completed_contexts.lock().unwrap();

            for (index, context) in self.contexts.iter().enumerate() {
                let progress = self.progress.get(index); // Clone arcs
                let progress_handle = ProgressHandle::new(progress, self.progress.clone());
                // If we've done this one already, skip it
                if completed_lock.contains(&index) {
                    progress_handle.add(context.length);
                    continue;
                }

                let context = context.clone();
                let control_flag = self.control_flag.clone(); // Clone arcs
                let completed_indexes_ref = completed_indexes_loop_arc.clone();

                scope.spawn(move |_| {
                    match download_game_chunk(context.clone(), control_flag, progress_handle) {
                        Ok(res) => {
                            if res {
                                let mut lock = completed_indexes_ref.lock().unwrap();
                                lock.push(index);
                            }
                        }
                        Err(e) => {
                            error!("GameDownloadError: {}", e);
                            self.sender.send(DownloadManagerSignal::Error(e)).unwrap();
                        }
                    }
                });
            }
        });

        let completed_lock_len = {
            let mut completed_lock = self.completed_contexts.lock().unwrap();
            let newly_completed_lock = completed_indexes.lock().unwrap();

            completed_lock.extend(newly_completed_lock.iter());

            completed_lock.len()
        };

        // If we're not out of contexts, we're not done, so we don't fire completed
        if completed_lock_len != self.contexts.len() {
            info!("da for {} exited without completing", self.id.clone());
            self.stored_manifest
                .set_completed_contexts(&self.completed_contexts);
            info!("Setting completed contexts");
            self.stored_manifest.write();
            info!("Wrote completed contexts");
            return Ok(());
        }

        // We've completed
        self.sender
            .send(DownloadManagerSignal::Completed(self.id.clone()))
            .unwrap();

        Ok(())
    }
}
