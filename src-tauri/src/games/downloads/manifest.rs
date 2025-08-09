use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct DownloadBucket {
    pub game_id: String,
    pub version: String,
    pub drops: Vec<DownloadDrop>,
}

#[derive(Deserialize)]
pub struct DownloadContext {
    pub context: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkBodyFile {
    filename: String,
    chunk_index: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkBody {
    pub context: String,
    pub files: Vec<ChunkBodyFile>,
}

#[derive(Serialize)]
pub struct ManifestBody {
    pub game: String,
    pub version: String,
}

impl ChunkBody {
    pub fn create(context: &DownloadContext, drops: &[DownloadDrop]) -> ChunkBody {
        Self {
            context: context.context.clone(),
            files: drops
                .iter()
                .map(|e| ChunkBodyFile {
                    filename: e.filename.clone(),
                    chunk_index: e.index,
                })
                .collect(),
        }
    }
}

pub type DropManifest = HashMap<String, DropChunk>;
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DropChunk {
    pub permissions: u32,
    pub ids: Vec<String>,
    pub checksums: Vec<String>,
    pub lengths: Vec<usize>,
    pub version_name: String,
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
