use bitcode::{Decode, Encode};
use serde_with::serde_as;
use std::{collections::HashMap, path::PathBuf};

use crate::{models::v1, platform::Platform};

use super::{Deserialize, Serialize, native_model};

fn default_template() -> String {
    "{}".to_owned()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[native_model(id = 2, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
pub struct GameVersion {
    pub game_id: String,
    pub version_name: String,

    pub platform: Platform,

    pub launch_command: String,
    pub launch_args: Vec<String>,
    #[serde(default = "default_template")]
    pub launch_command_template: String,

    pub setup_command: String,
    pub setup_args: Vec<String>,
    #[serde(default = "default_template")]
    pub setup_command_template: String,

    pub only_setup: bool,

    pub version_index: usize,
    pub delta: bool,

    pub umu_id_override: Option<String>,
}

#[serde_as]
#[derive(Serialize, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[native_model(id = 3, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
pub struct DatabaseApplications {
    pub install_dirs: Vec<PathBuf>,
    // Guaranteed to exist if the game also exists in the app state map
    pub game_statuses: HashMap<String, v1::GameDownloadStatus>,
    pub game_versions: HashMap<String, HashMap<String, v1::GameVersion>>,
    pub installed_game_version: HashMap<String, v1::DownloadableMetadata>,

    #[serde(skip)]
    pub transient_statuses: HashMap<v1::DownloadableMetadata, v1::ApplicationTransientStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[native_model(id = 4, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
pub struct Settings {
    pub autostart: bool,
    pub max_download_threads: usize,
    pub force_offline: bool, // ... other settings ...
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            max_download_threads: 4,
            force_offline: false,
        }
    }
}

// Strings are version names for a particular game
#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(tag = "type")]
#[native_model(id = 5, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
pub enum GameDownloadStatus {
    Remote {},
    SetupRequired {
        version_name: String,
        install_dir: String,
    },
    Installed {
        version_name: String,
        install_dir: String,
    },
}

// Stuff that shouldn't be synced to disk
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ApplicationTransientStatus {
    Queued { version_name: String },
    Downloading { version_name: String },
    Uninstalling {},
    Updating { version_name: String },
    Validating { version_name: String },
    Running {},
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[native_model(id = 6, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
pub struct DatabaseAuth {
    pub private: String,
    pub cert: String,
    pub client_id: String,
    pub web_token: Option<String>,
}

#[native_model(id = 8, version = 1)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, Copy)]
pub enum DownloadType {
    Game,
    Tool,
    Dlc,
    Mod,
}

#[native_model(id = 7, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Debug, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadableMetadata {
    pub id: String,
    pub version: Option<String>,
    pub download_type: v1::DownloadType,
}
impl DownloadableMetadata {
    pub fn new(id: String, version: Option<String>, download_type: v1::DownloadType) -> Self {
        Self {
            id,
            version,
            download_type,
        }
    }
}

#[native_model(id = 1, version = 1)]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Database {
    #[serde(default)]
    pub settings: Settings,
    pub auth: Option<v1::DatabaseAuth>,
    pub base_url: String,
    pub applications: v1::DatabaseApplications,
    pub prev_database: Option<PathBuf>,
    pub cache_dir: PathBuf,
}
#[native_model(id = 15, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode, Default)]
pub struct LibraryMetadata {
    pub(crate) providers: Vec<v1::LibraryProviderMetadata>
}


#[native_model(id = 11, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct LibraryProviderMetadata {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) provider: v1::ProviderType
}
#[native_model(id = 10, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub enum ProviderType {
    Drop(v1::DatabaseAuth),
}
#[native_model(id = 12, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
pub struct Game {
    pub library_id: LibraryProviderMetadata,
    pub(crate) id: String,
    m_name: String,
    m_short_description: String,
    m_description: String,
    // mDevelopers
    // mPublishers
    m_icon_object_id: String,
    m_banner_object_id: String,
    m_cover_object_id: String,
    m_image_library_object_ids: Vec<String>,
    m_image_carousel_object_ids: Vec<String>,
}

#[native_model(id = 13, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    id: String,
    name: String,
    is_default: bool,
    user_id: String,
    entries: Vec<CollectionObject>,
}

#[native_model(id = 14, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionObject {
    collection_id: String,
    game_id: String,
    game: Game,
}
