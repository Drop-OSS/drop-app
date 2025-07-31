use crate::database::models::data::Database;

pub mod data {
    use std::path::PathBuf;

    use native_model::native_model;
    use serde::{Deserialize, Serialize};

    pub type GameVersion = v1::GameVersion;
    pub type Database = v3::Database;
    pub type Settings = v1::Settings;
    pub type DatabaseAuth = v1::DatabaseAuth;

    pub type GameDownloadStatus = v2::GameDownloadStatus;
    pub type ApplicationTransientStatus = v1::ApplicationTransientStatus;
    pub type DownloadableMetadata = v1::DownloadableMetadata;
    pub type DownloadType = v1::DownloadType;
    pub type DatabaseApplications = v2::DatabaseApplications;
    pub type DatabaseCompatInfo = v2::DatabaseCompatInfo;

    use std::{collections::HashMap, process::Command};

    use crate::process::process_manager::UMU_LAUNCHER_EXECUTABLE;

    pub mod v1 {
        use crate::process::process_manager::Platform;
        use serde_with::serde_as;
        use std::{collections::HashMap, path::PathBuf};

        use super::{Serialize, Deserialize, native_model};

        fn default_template() -> String {
            "{}".to_owned()
        }

        #[derive(Serialize, Deserialize, Clone, Debug)]
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
            pub game_statuses: HashMap<String, GameDownloadStatus>,
            pub game_versions: HashMap<String, HashMap<String, GameVersion>>,
            pub installed_game_version: HashMap<String, DownloadableMetadata>,

            #[serde(skip)]
            pub transient_statuses: HashMap<DownloadableMetadata, ApplicationTransientStatus>,
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
        #[derive(Serialize, Clone, Deserialize)]
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
            Downloading { version_name: String },
            Uninstalling {},
            Updating { version_name: String },
            Running {},
        }

        #[derive(serde::Serialize, Clone, Deserialize)]
        #[native_model(id = 6, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        pub struct DatabaseAuth {
            pub private: String,
            pub cert: String,
            pub client_id: String,
            pub web_token: Option<String>,
        }

        #[native_model(id = 8, version = 1)]
        #[derive(
            Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, Copy,
        )]
        pub enum DownloadType {
            Game,
            Tool,
            Dlc,
            Mod,
        }

        #[native_model(id = 7, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct DownloadableMetadata {
            pub id: String,
            pub version: Option<String>,
            pub download_type: DownloadType,
        }
        impl DownloadableMetadata {
            pub fn new(id: String, version: Option<String>, download_type: DownloadType) -> Self {
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
            pub auth: Option<DatabaseAuth>,
            pub base_url: String,
            pub applications: DatabaseApplications,
            pub prev_database: Option<PathBuf>,
            pub cache_dir: PathBuf,
        }
    }

    pub mod v2 {
        use std::{collections::HashMap, path::PathBuf};

        use serde_with::serde_as;

        use super::{Serialize, Deserialize, native_model, Settings, DatabaseAuth, v1, GameVersion, DownloadableMetadata, ApplicationTransientStatus};

        #[native_model(id = 1, version = 2, with = native_model::rmp_serde_1_3::RmpSerde)]
        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct Database {
            #[serde(default)]
            pub settings: Settings,
            pub auth: Option<DatabaseAuth>,
            pub base_url: String,
            pub applications: v1::DatabaseApplications,
            #[serde(skip)]
            pub prev_database: Option<PathBuf>,
            pub cache_dir: PathBuf,
            pub compat_info: Option<DatabaseCompatInfo>,
        }

        #[native_model(id = 8, version = 2, with = native_model::rmp_serde_1_3::RmpSerde)]
        #[derive(Serialize, Deserialize, Clone, Default)]

        pub struct DatabaseCompatInfo {
            pub umu_installed: bool,
        }

        impl From<v1::Database> for Database {
            fn from(value: v1::Database) -> Self {
                Self {
                    settings: value.settings,
                    auth: value.auth,
                    base_url: value.base_url,
                    applications: value.applications,
                    prev_database: value.prev_database,
                    cache_dir: value.cache_dir,
                    compat_info: crate::database::models::Database::create_new_compat_info(),
                }
            }
        }
        // Strings are version names for a particular game
        #[derive(Serialize, Clone, Deserialize, Debug)]
        #[serde(tag = "type")]
        #[native_model(id = 5, version = 2, with = native_model::rmp_serde_1_3::RmpSerde)]
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
            PartiallyInstalled {
                version_name: String,
                install_dir: String,
            },
        }
        impl From<v1::GameDownloadStatus> for GameDownloadStatus {
            fn from(value: v1::GameDownloadStatus) -> Self {
                match value {
                    v1::GameDownloadStatus::Remote {} => Self::Remote {},
                    v1::GameDownloadStatus::SetupRequired {
                        version_name,
                        install_dir,
                    } => Self::SetupRequired {
                        version_name,
                        install_dir,
                    },
                    v1::GameDownloadStatus::Installed {
                        version_name,
                        install_dir,
                    } => Self::Installed {
                        version_name,
                        install_dir,
                    },
                }
            }
        }
        #[serde_as]
        #[derive(Serialize, Clone, Deserialize, Default)]
        #[serde(rename_all = "camelCase")]
        #[native_model(id = 3, version = 2, with = native_model::rmp_serde_1_3::RmpSerde)]
        pub struct DatabaseApplications {
            pub install_dirs: Vec<PathBuf>,
            // Guaranteed to exist if the game also exists in the app state map
            pub game_statuses: HashMap<String, GameDownloadStatus>,
            pub game_versions: HashMap<String, HashMap<String, GameVersion>>,
            pub installed_game_version: HashMap<String, DownloadableMetadata>,

            #[serde(skip)]
            pub transient_statuses: HashMap<DownloadableMetadata, ApplicationTransientStatus>,
        }
        impl From<v1::DatabaseApplications> for DatabaseApplications {
            fn from(value: v1::DatabaseApplications) -> Self {
                Self {
                    game_statuses: value
                        .game_statuses
                        .into_iter()
                        .map(|x| (x.0, x.1.into()))
                        .collect::<HashMap<String, GameDownloadStatus>>(),
                    install_dirs: value.install_dirs,
                    game_versions: value.game_versions,
                    installed_game_version: value.installed_game_version,
                    transient_statuses: value.transient_statuses,
                }
            }
        }
    }
    mod v3 {
        use std::path::PathBuf;

        use super::{Serialize, Deserialize, native_model, Settings, DatabaseAuth, DatabaseApplications, DatabaseCompatInfo, v2};
        #[native_model(id = 1, version = 3, with = native_model::rmp_serde_1_3::RmpSerde)]
        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct Database {
            #[serde(default)]
            pub settings: Settings,
            pub auth: Option<DatabaseAuth>,
            pub base_url: String,
            pub applications: DatabaseApplications,
            #[serde(skip)]
            pub prev_database: Option<PathBuf>,
            pub cache_dir: PathBuf,
            pub compat_info: Option<DatabaseCompatInfo>,
        }
        impl From<v2::Database> for Database {
            fn from(value: v2::Database) -> Self {
                Self {
                    settings: value.settings,
                    auth: value.auth,
                    base_url: value.base_url,
                    applications: value.applications.into(),
                    prev_database: value.prev_database,
                    cache_dir: value.cache_dir,
                    compat_info: Database::create_new_compat_info(),
                }
            }
        }
    }
    impl Database {
        fn create_new_compat_info() -> Option<DatabaseCompatInfo> {
            #[cfg(target_os = "windows")]
            return None;

            let has_umu_installed = Command::new(UMU_LAUNCHER_EXECUTABLE).spawn().is_ok();
            Some(DatabaseCompatInfo {
                umu_installed: has_umu_installed,
            })
        }

        pub fn new<T: Into<PathBuf>>(
            games_base_dir: T,
            prev_database: Option<PathBuf>,
            cache_dir: PathBuf,
        ) -> Self {
            Self {
                applications: DatabaseApplications {
                    install_dirs: vec![games_base_dir.into()],
                    game_statuses: HashMap::new(),
                    game_versions: HashMap::new(),
                    installed_game_version: HashMap::new(),
                    transient_statuses: HashMap::new(),
                },
                prev_database,
                base_url: String::new(),
                auth: None,
                settings: Settings::default(),
                cache_dir,
                compat_info: Database::create_new_compat_info(),
            }
        }
    }
}
