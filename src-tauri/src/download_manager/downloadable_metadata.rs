use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
pub enum DownloadType {
    Game,
    Tool,
    DLC,
    Mod
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
pub struct DownloadableMetadata {
    pub id: String,
    pub version: String,
    pub download_type: DownloadType
}
impl DownloadableMetadata {
    pub fn new(id: String, version: String, download_type: DownloadType) -> Self {
        Self {
            id,
            version,
            download_type
        }
    }
}