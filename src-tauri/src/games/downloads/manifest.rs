use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
pub struct DropDownloadContext {
    pub file_name: String,
    pub version: String,
    pub index: usize,
    pub offset: u64,
    pub game_id: String,
    pub path: PathBuf,
    pub checksum: String,
    pub length: usize,
    pub permissions: u32,
}
