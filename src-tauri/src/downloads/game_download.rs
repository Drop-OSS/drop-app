use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use serde::{Deserialize, Serialize};
use versions::Version;
use crate::AppState;
use crate::downloads::progress::ProgressChecker;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct GameDownload {
    id: String,
    version: Version,
    progress: GameDownloadState
}
#[derive(Serialize, Deserialize, Clone)]
pub enum GameDownloadState {
    Uninitialised,
    Manifest,
    Downloading(Arc<AtomicUsize>),
    Finished,
    Stalled,
    Failed,
    Cancelled
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameDownloadError {

}
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct GameChunkCtx {
    chunk_id: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameDownloadManifest {
    // TODO: Implement game manifest
}

impl GameDownload {
    pub fn new(id: String, version: Version) -> Self {
        Self {
            id,
            version,
            progress: GameDownloadState::Uninitialised
        }
    }
    pub async fn download(&mut self, max_threads: usize, contexts: Vec<GameChunkCtx>) -> Result<(), GameDownloadError> {
        let progress = Arc::new(AtomicUsize::new(0));
        self.progress = GameDownloadState::Downloading(progress.clone());
        let progress = ProgressChecker::new(Box::new(download_game_chunk), progress);
        progress.run_contexts_parallel_async(contexts, max_threads).await;
        Ok(())
    }
    pub async fn download_manifest(&mut self) -> Result<GameDownloadManifest, GameDownloadError> {
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
    game_version: Version,
    max_threads: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), GameDownloadError> {
    let mut download = Arc::new(GameDownload::new(game_id, game_version));
    let mut app_state = state.lock().unwrap();
    app_state.game_downloads.push(download.clone());

    let manifest = match download.download_manifest().await {
        Ok(manifest) => { manifest }
        Err(e) => { return Err(e) }
    };

    download.download(max_threads, manifest.parse_to_chunks()).await
}

impl GameDownloadManifest {
    fn parse_to_chunks(self) -> Vec<GameChunkCtx> {
        todo!()
    }
}