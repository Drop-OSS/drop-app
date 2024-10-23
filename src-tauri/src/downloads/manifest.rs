use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub type DropManifest = HashMap<String, DropChunk>;
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DropChunk {
    pub permissions: usize,
    pub ids: Vec<String>,
    pub checksums: Vec<String>,
    pub lengths: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DropDownloadContext {
    pub file_chunk: Arc<DropChunk>,
    pub file_name: String,
    pub version: String,
    pub index: usize
}