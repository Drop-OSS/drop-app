use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use versions::Version;
use crate::downloads::progress::ProgressChecker;

pub struct GameDownload {
    id: String,
    version: Version,
    progress: Arc<AtomicUsize>
}
pub struct GameChunkCtx {
    
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
        progress.run_contexts_sequentially_async(contexts).await;
    }
}
fn download_game_chunk(ctx: GameChunkCtx) {
    todo!()
}