use std::future::Future;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use log::info;
use serde::{Deserialize, Serialize};
use versions::Version;
use crate::{AppState, DB};
use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::progress::ProgressChecker;

#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct GameDownload {
    id: String,
    version: Version,
    progress: Arc<AtomicUsize>,
    state: Mutex<GameDownloadState>,
    manifest: Option<Mutex<GameDownloadManifest>>
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
    Cancelled
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum GameDownloadError {
    ManifestAlreadyExists,
    ManifestDoesNotExist,
    ManifestDownloadError,
    StatusError(u16)
}
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all="camelCase")]
pub struct GameChunkCtx {
    chunk_id: usize,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct GameDownloadManifest {
    // TODO: Implement game manifest
}

impl GameDownload {
    pub fn new(id: String, version: Version) -> Self {
        Self {
            id,
            version,
            progress: Arc::new(AtomicUsize::new(0)),
            state: Mutex::from(GameDownloadState::Uninitialised),
            manifest: None
        }
    }
    pub async fn queue(&self) -> Result<(), GameDownloadError> {
        self.change_state(GameDownloadState::Queued);
        if self.manifest.is_none() {
            return Ok(())
        }
        self.download_manifest().await
    }
    pub async fn download(&self, max_threads: usize, contexts: Vec<GameChunkCtx>) -> Result<(), GameDownloadError> {
        let progress = Arc::new(AtomicUsize::new(0));
        self.change_state(GameDownloadState::Downloading);
        let progress = ProgressChecker::new(Box::new(download_game_chunk), progress);
        progress.run_contexts_parallel_async(contexts, max_threads).await;
        Ok(())
    }
    pub async fn download_manifest(&self) -> Result<(), GameDownloadError> {
        if self.manifest.is_some() {
            return Err(GameDownloadError::ManifestAlreadyExists);
        }

        info!("Getting url components");
        let base_url = DB.fetch_base_url();
        let manifest_url = base_url
            .join(
                format!(
                    "/api/v1/client/metadata/manifest?id={}&version={}",
                    self.id,
                    self.version.to_string()
                )
                .as_str()
            )
            .unwrap();

        info!("Generating authorization header");
        let header = generate_authorization_header();

        info!("Generating & sending client");
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(manifest_url.to_string())
            .header("Authorization", header)
            .send()
            .unwrap();

        info!("Got status");
        if response.status() != 200 {
            return Err(GameDownloadError::StatusError(response.status().as_u16()));
        }

        info!("{:?}", response.text());

        Ok(())
    }
    pub fn change_state(&self, state: GameDownloadState) {
        let mut lock = self.state.lock().unwrap();
        *lock = state;
    }
}
impl GameDownloadManifest {
    fn parse_to_chunks(&self) -> Vec<GameChunkCtx> {
        todo!()
    }
}
fn download_game_chunk(ctx: GameChunkCtx) {
    todo!();
    // Need to implement actual download logic
}

#[tauri::command]
pub async fn start_game_download(
    game_id: String,
    game_version: String,
    max_threads: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), GameDownloadError> {

    info!("Triggered Game Download");

    let mut download = Arc::new(GameDownload::new(game_id, Version::from_str(&*game_version).unwrap()));
    //let mut app_state = state.lock().unwrap();

    let tmp = download.clone();
    //let manifest = &tmp.manifest;

    let res = download.download_manifest().await;

    res
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

