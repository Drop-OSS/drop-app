use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

pub type DropManifest = HashMap<String, DropChunk>;
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DropChunk {
    pub permissions: usize,
    pub ids: Vec<String>,
    pub checksums: Vec<String>,
    pub lengths: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DropDownloadContext {
    pub file_name: String,
    pub version: String,
    pub index: usize,
    pub offset: u64,
    pub game_id: String,
    pub file: Arc<Mutex<File>>
}