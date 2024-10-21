use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type DropManifest = HashMap<String, DropChunk>;
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DropChunk {
    permissions: usize,
    ids: Vec<String>,
    checksums: Vec<String>,
    lengths: Vec<usize>
}