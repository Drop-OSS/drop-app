use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
// Drops go in buckets
pub struct DownloadDrop {
    pub index: usize,
    pub filename: String,
    pub path: PathBuf,
    pub start: usize,
    pub length: usize,
    pub checksum: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadBucket {
    pub game_id: String,
    pub version: String,
    pub drops: Vec<DownloadDrop>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DropValidateContext {
    pub index: usize,
    pub offset: usize,
    pub path: PathBuf,
    pub checksum: String,
    pub length: usize,
}

impl From<DownloadBucket> for Vec<DropValidateContext> {
    fn from(value: DownloadBucket) -> Self {
        value
            .drops
            .into_iter()
            .map(|e| DropValidateContext {
                index: e.index,
                offset: e.start,
                path: e.path,
                checksum: e.checksum,
                length: e.length,
            })
            .collect()
    }
}
