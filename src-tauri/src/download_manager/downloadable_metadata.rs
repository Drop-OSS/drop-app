use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, Copy)]
pub enum DownloadType {
    Game,
    Tool,
    DLC,
    Mod
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadableMetadata {
    pub id: String,
    pub version: Option<String>,
    pub download_type: DownloadType
}
impl DownloadableMetadata {
    pub fn new(id: String, version: Option<String>, download_type: DownloadType) -> Self {
        Self {
            id,
            version,
            download_type
        }
    }
}