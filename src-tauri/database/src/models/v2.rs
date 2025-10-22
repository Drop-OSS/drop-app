        use std::{collections::HashMap, path::PathBuf};

        use serde_with::serde_as;

        use super::{Deserialize, Serialize, native_model, v1};

        #[native_model(id = 1, version = 2, with = native_model::rmp_serde_1_3::RmpSerde, from = v1::Database)]
        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct Database {
            #[serde(default)]
            pub settings: v1::Settings,
            pub auth: Option<v1::DatabaseAuth>,
            pub base_url: String,
            pub applications: v1::DatabaseApplications,
            #[serde(skip)]
            pub prev_database: Option<PathBuf>,
            pub cache_dir: PathBuf,
            pub compat_info: Option<DatabaseCompatInfo>,
        }

        #[native_model(id = 9, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
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
                    compat_info: None,
                }
            }
        }
        // Strings are version names for a particular game
        #[derive(Serialize, Clone, Deserialize, Debug)]
        #[serde(tag = "type")]
        #[native_model(id = 5, version = 2, with = native_model::rmp_serde_1_3::RmpSerde, from = v1::GameDownloadStatus)]
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
        #[native_model(id = 3, version = 2, with = native_model::rmp_serde_1_3::RmpSerde, from=v1::DatabaseApplications)]
        pub struct DatabaseApplications {
            pub install_dirs: Vec<PathBuf>,
            // Guaranteed to exist if the game also exists in the app state map
            pub game_statuses: HashMap<String, GameDownloadStatus>,

            pub game_versions: HashMap<String, HashMap<String, v1::GameVersion>>,
            pub installed_game_version: HashMap<String, v1::DownloadableMetadata>,

            #[serde(skip)]
            pub transient_statuses:
                HashMap<v1::DownloadableMetadata, v1::ApplicationTransientStatus>,
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
