use crate::database::db::GameVersion;

use super::conditions::{Condition};


#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CloudSaveMetadata {
    pub files: Vec<GameFile>,
    pub game_version: GameVersion,
    pub save_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameFile {
    pub path: String,
    pub id: Option<String>,
    pub data_type: DataType,
    pub tags: Vec<Tag>,
    pub conditions: Vec<Condition>
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum DataType {
    Registry,
    File,
    Other
}
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tag {
    Config,
    Save,
    #[default]
    #[serde(other)]
    Other,
}