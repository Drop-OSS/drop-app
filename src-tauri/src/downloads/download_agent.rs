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
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

pub struct GameDownloadAgent {
    id: String,
    version: String,
    state: Mutex<GameDownloadState>,
    contexts: Mutex<Vec<DropDownloadContext>>,
    progress: ProgressChecker<DropDownloadContext>,
    pub manifest: Mutex<Option<DropManifest>>,
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
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct GameChunkCtx {
    chunk_id: usize,
}
impl GameDownloadAgent {
    pub fn new(id: String, version: String) -> Self {
        Self {
            id,
            version,
            state: Mutex::from(GameDownloadState::Uninitialised),
            manifest: Mutex::new(None),
            progress: ProgressChecker::new(
                Box::new(download_logic::download_game_chunk),
                Arc::new(AtomicUsize::new(0)),
            ),
            contexts: Mutex::new(Vec::new()),
        }
    }
    pub async fn queue(&self) -> Result<(), GameDownloadError> {
        self.change_state(GameDownloadState::Queued);
        if self.manifest.lock().unwrap().is_none() {
            return Ok(());
        }
        self.ensure_manifest_exists().await
    }

    pub fn begin_download(&self, max_threads: usize) -> Result<(), GameDownloadError> {
        self.change_state(GameDownloadState::Downloading);
        // TODO we're coping the whole context thing
        // It's not necessary, I just can't figure out to make the borrow checker happy
        self.progress
            .run_contexts_parallel(self.contexts.lock().unwrap().to_vec(), max_threads);
        Ok(())
    }

    pub async fn ensure_manifest_exists(&self) -> Result<(), GameDownloadError> {
        if self.manifest.lock().unwrap().is_some() {
            return Ok(());
        }

        self.download_manifest().await
    }

    async fn download_manifest(&self) -> Result<(), GameDownloadError> {
        let base_url = DB.fetch_base_url();
        let manifest_url = base_url
            .join(
                format!(
                    "/api/v1/client/metadata/manifest?id={}&version={}",
                    encode(&self.id), encode(&self.version)
                )
                .as_str(),
            )
            .unwrap();

        let header = generate_authorization_header();

        info!("Generating & sending client");
        let client = reqwest::Client::new();
        let response = client
            .get(manifest_url.to_string())
            .header("Authorization", header)
            .send()
            .await
            .unwrap();

        if response.status() != 200 {
            info!("Error status: {}", response.status());
            return Err(GameDownloadError::Status(response.status().as_u16()));
        }

        let manifest_download = response.json::<DropManifest>().await.unwrap();
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

#[tauri::command]
pub async fn start_game_download(
    game_id: String,
    game_version: String,
    max_threads: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), GameDownloadError> {
    info!("Triggered Game Download");

    let download_agent = GameDownloadAgent::new(game_id.clone(), game_version.clone());

    download_agent.ensure_manifest_exists().await?;

    let local_manifest = {
        let manifest = download_agent.manifest.lock().unwrap();
        (*manifest).clone().unwrap()
    };

    download_agent.generate_job_contexts(&local_manifest, game_version.clone(), game_id).unwrap();

    download_agent.begin_download(max_threads)?;

    Ok(())
}
