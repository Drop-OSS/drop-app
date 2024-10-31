use crate::auth::generate_authorization_header;
use crate::db::{DatabaseImpls, DATA_ROOT_DIR};
use crate::downloads::download_logic;
use crate::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::downloads::progress::ProgressChecker;
use crate::{AppState, DB};
use log::info;
use rustix::fs::{fallocate, FallocateFlags};
use serde::{Deserialize, Serialize};
use urlencoding::encode;
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};

pub struct GameDownloadAgent {
    pub id: String,
    pub version: String,
    state: Mutex<GameDownloadState>,
    contexts: Mutex<Vec<DropDownloadContext>>,
    progress: ProgressChecker<DropDownloadContext>,
    pub manifest: Mutex<Option<DropManifest>>,
    pub callback: Arc<AtomicBool>
}
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum GameDownloadState {
    Uninitialised,
    Queued,
    Manifest,
    Downloading,
    Finished,
    Stalled,
    Failed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum GameDownloadError {
    ManifestDownload,
    FailedContextGeneration,
    Status(u16),
    System(SystemError),
}
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum SystemError {
    MutexLockFailed,
}

impl GameDownloadAgent {
    pub fn new(id: String, version: String) -> Self {
        let callback = Arc::new(AtomicBool::new(false));
        Self {
            id,
            version,
            state: Mutex::from(GameDownloadState::Uninitialised),
            manifest: Mutex::new(None),
            callback: callback.clone(),
            progress: ProgressChecker::new(
                Box::new(download_logic::download_game_chunk),
                Arc::new(AtomicUsize::new(0)),
                callback
            ),
            contexts: Mutex::new(Vec::new()),
        }
    }
    pub async fn queue(&self) -> Result<(), GameDownloadError> {
        self.change_state(GameDownloadState::Queued);
        if self.manifest.lock().unwrap().is_none() {
            return Ok(());
        }
        self.ensure_manifest_exists()
    }

    pub fn begin_download(&self, max_threads: usize) -> Result<(), GameDownloadError> {
        self.change_state(GameDownloadState::Downloading);
        // TODO we're coping the whole context thing
        // It's not necessary, I just can't figure out to make the borrow checker happy
        {
            let lock = self.contexts.lock().unwrap().to_vec();
            self.progress
                .run_context_parallel(lock, max_threads);
        }
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
                    self.id, encode(&self.version)
                )
                .as_str(),
            )
            .unwrap();

        let header = generate_authorization_header();

        info!("Generating & sending client");
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(manifest_url.to_string())
            .header("Authorization", header)
            .send()
            .unwrap();

        if response.status() != 200 {
            info!("Error status: {}", response.status());
            return Err(GameDownloadError::Status(response.status().as_u16()));
        }

        let manifest_download = response.json::<DropManifest>().unwrap();
        if let Ok(mut manifest) = self.manifest.lock() {
            *manifest = Some(manifest_download)
        } else {
            return Err(GameDownloadError::System(SystemError::MutexLockFailed));
        }

        Ok(())
    }

    pub fn change_state(&self, state: GameDownloadState) {
        let mut lock = self.state.lock().unwrap();
        *lock = state;
    }
    pub fn get_state(&self) -> GameDownloadState {
        let lock = self.state.lock().unwrap();
        lock.clone()
    }

    pub fn generate_job_contexts(
        &self,
        manifest: &DropManifest,
        version: String,
        game_id: String,
    ) -> Result<(), GameDownloadError> {
        let mut contexts = Vec::new();
        let base_path = DATA_ROOT_DIR.join("games").join(game_id.clone()).clone();
        create_dir_all(base_path.clone()).unwrap();
        info!("Generating contexts");
        for (raw_path, chunk) in manifest {
            let path = base_path.join(Path::new(raw_path));

            let container = path.parent().unwrap();
            create_dir_all(container).unwrap();

            let file = File::create(path.clone()).unwrap();
            let mut running_offset = 0;

            for (i, length) in chunk.lengths.iter().enumerate() {
                contexts.push(DropDownloadContext {
                    file_name: raw_path.to_string(),
                    version: version.to_string(),
                    offset: running_offset,
                    index: i,
                    game_id: game_id.to_string(),
                    path: path.clone(),
                    checksum: chunk.checksums[i].clone()
                });
                running_offset += *length as u64;
            }

            if running_offset > 0 {
                fallocate(file, FallocateFlags::empty(), 0, running_offset).unwrap();
            }
        }
        info!("Finished generating");
        if let Ok(mut context_lock) = self.contexts.lock() {
            *context_lock = contexts;
        } else {
            return Err(GameDownloadError::FailedContextGeneration);
        }

        Ok(())
    }
}

