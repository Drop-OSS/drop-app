use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub type DropManifest = HashMap<String, DropChunk>;
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DropChunk {
    pub permissions: usize,
    pub ids: Vec<String>,
    pub checksums: Vec<String>,
    pub lengths: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DropDownloadContext {
    pub file_name: String,
    pub version: String,
    pub index: usize,
    pub offset: u64,
    pub game_id: String,
    pub path: PathBuf,
    pub checksum: String,
    pub length: usize
}
