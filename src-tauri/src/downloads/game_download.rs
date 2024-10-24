use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use log::info;
use serde::{Deserialize, Serialize};
use crate::{AppState, DB};
use crate::auth::generate_authorization_header;
use crate::db::{DatabaseImpls, DATA_ROOT_DIR};
use crate::downloads::download_files;
use crate::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::downloads::progress::ProgressChecker;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDownload {
    id: String,
    version: String,
    progress: Arc<AtomicUsize>,
    state: Mutex<GameDownloadState>,
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
    Status(u16),
    System(SystemError),
}
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum SystemError {
    MutexLockFailed
}
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct GameChunkCtx {
    chunk_id: usize,
}
impl GameDownload {
    pub fn new(id: String, version: String) -> Self {
        Self {
            id,
            version,
            progress: Arc::new(AtomicUsize::new(0)),
            state: Mutex::from(GameDownloadState::Uninitialised),
            manifest: Mutex::new(None),
        }
    }
    pub async fn queue(&self) -> Result<(), GameDownloadError> {
        self.change_state(GameDownloadState::Queued);
        if self.manifest.lock().unwrap().is_none() {
            return Ok(());
        }
        self.ensure_manifest_exists().await
    }
    pub async fn download(&self, max_threads: usize, contexts: Vec<DropDownloadContext>) -> Result<(), GameDownloadError> {
        let progress = Arc::new(AtomicUsize::new(0));
        self.change_state(GameDownloadState::Downloading);
        let progress = ProgressChecker::new(Box::new(download_files::download_game_chunk), progress);
        progress.run_contexts_parallel_async(contexts, max_threads).await;
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
                    self.id,
                    self.version
                )
                    .as_str()
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
        info!("Manifest: {:?}", manifest_download);
        if let Ok(mut manifest) = self.manifest.lock() {
            *manifest = Some(manifest_download)
        } else { return Err(GameDownloadError::System(SystemError::MutexLockFailed)); }

        Ok(())
    }

    pub fn change_state(&self, state: GameDownloadState) {
        let mut lock = self.state.lock().unwrap();
        *lock = state;
    }
}
pub fn to_contexts(manifest: &DropManifest, version: String, game_id: String) -> Vec<DropDownloadContext> {
    let mut contexts = Vec::new();
    let base_path = DATA_ROOT_DIR.clone();
    for key in manifest {
        let path = base_path.join(Path::new(key.0));
        let file = Arc::new(Mutex::new(File::create(path).unwrap()));
        for i in 0..key.1.ids.len() {
            contexts.push(DropDownloadContext {
                file_chunk: Arc::new(key.1.clone()),

                file_name: key.0.clone(),
                version: version.to_string(),
                index: i,
                game_id: game_id.to_string(),
                file: file.clone(),
            });

        }
    }
    info!("Contexts: {:?}", contexts);
    contexts
}

#[tauri::command]
pub async fn start_game_download(
    game_id: String,
    game_version: String,
    max_threads: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), GameDownloadError> {
    info!("Triggered Game Download");

    let download = Arc::new(GameDownload::new(game_id.clone(), game_version.clone()));

    download.ensure_manifest_exists().await?;

    let local_manifest = {
        let manifest = download.manifest.lock().unwrap();
        (*manifest).clone().unwrap()
    };

    let contexts = to_contexts(&local_manifest, game_version.clone(), game_id);

    let _ = download.download(max_threads, contexts).await;

    Ok(())

    /*
    let Some(unlocked) = manifest else { return Err(GameDownloadError::ManifestDoesNotExist) };
    let lock = unlocked.lock().unwrap();

    let chunks = lock.parse_to_chunks();

    /*
    let manifest = match d.manifest {
        Some(lock) => {
            let lock = lock.lock().unwrap();
            lock.parse_to_chunks()
        },
        None => { return Err(GameDownloadError::ManifestDoesNotExist) }
    };
     */

    app_state.game_downloads.push(download.clone());
    download.download(max_threads, chunks).await

     */
}

