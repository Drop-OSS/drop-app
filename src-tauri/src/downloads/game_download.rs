use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use serde::{Deserialize, Serialize};
use versions::Version;
use crate::downloads::progress::ProgressChecker;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct GameDownload {
    id: String,
    version: Version,
    progress: Arc<AtomicUsize>
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct GameChunkCtx {
    chunk_id: usize,
}

impl GameDownload {
    pub fn new(id: String, version: Version) -> Self {
        Self {
            id,
            version,
            progress: Arc::new(AtomicUsize::new(0))
        }
    }
    pub async fn download(&self, max_threads: usize, contexts: Vec<GameChunkCtx>) {
        let progress = ProgressChecker::new(Box::new(download_game_chunk), self.progress.clone());
        progress.run_contexts_parallel_async(contexts, max_threads).await;
    }
}
fn download_game_chunk(ctx: GameChunkCtx) {
    todo!();
    // Need to implement actual download logic
}