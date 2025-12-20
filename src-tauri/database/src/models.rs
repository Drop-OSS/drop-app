pub mod data {
    use std::{hash::Hash, path::PathBuf};

    use native_model::native_model;
    use serde::{Deserialize, Serialize};

    // NOTE: Within each version, you should NEVER use these types.
    // Declare it using the actual version that it is from, i.e. v1::Settings rather than just Settings from here

    pub type GameVersion = v1::GameVersion;
    pub type Database = v1::Database;
    pub type Settings = v1::Settings;
    pub type DatabaseAuth = v1::DatabaseAuth;

    pub type GameDownloadStatus = v1::GameDownloadStatus;
    pub type ApplicationTransientStatus = v1::ApplicationTransientStatus;
    /**
     * Need to be universally accessible by the ID, and the version is just a couple sprinkles on top
     */
    pub type DownloadableMetadata = v1::DownloadableMetadata;
    pub type DownloadType = v1::DownloadType;
    pub type DatabaseApplications = v1::DatabaseApplications;

    use std::collections::HashMap;

    impl PartialEq for DownloadableMetadata {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id && self.download_type == other.download_type
        }
    }
    impl Hash for DownloadableMetadata {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state);
            self.download_type.hash(state);
        }
    }

    mod v1 {
        use serde_with::serde_as;
        use std::{collections::HashMap, path::PathBuf};

        use crate::platform::Platform;

        use super::{Deserialize, Serialize, native_model};

        fn default_template() -> String {
            "{}".to_owned()
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        #[native_model(id = 2, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        pub struct GameVersion {
            pub game_id: String,
            pub version_id: String,

            pub display_name: Option<String>,
            pub version_path: String,

            pub only_setup: bool,

            pub version_index: usize,
            pub delta: bool,

            #[serde(default = "default_template")]
            pub launch_template: String,

            pub launches: Vec<LaunchConfiguration>,
            pub setups: Vec<SetupConfiguration>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        #[native_model(id = 10, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        pub struct LaunchConfiguration {
            pub launch_id: String,

            pub name: String,
            pub command: String,
            pub args: Vec<String>,
            pub platform: Platform,
            pub umu_id_override: Option<String>,

            pub executor: Option<LaunchConfigurationExecutor>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        #[native_model(id = 11, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        /**
         * This is intended to be used to look up the actual launch configuration that we store elsewhere
         */

        pub struct LaunchConfigurationExecutor {
            pub launch_id: String,

            pub game_id: String,
            pub version_id: String,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        #[native_model(id = 12, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        pub struct SetupConfiguration {
            pub command: String,
            pub args: Vec<String>,
            pub platform: Platform,
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
        // Stuff that shouldn't be synced to disk
        #[derive(Clone, Serialize, Deserialize, Debug)]
        pub enum ApplicationTransientStatus {
            Queued { version_id: String },
            Downloading { version_id: String },
            Uninstalling {},
            Updating { version_id: String },
            Validating { version_id: String },
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
        #[derive(Debug, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct DownloadableMetadata {
            pub id: String,
            pub version: String,
            pub target_platform: Platform,
            pub download_type: DownloadType,
        }
        impl DownloadableMetadata {
            pub fn new(
                id: String,
                version: String,
                target_platform: Platform,
                download_type: DownloadType,
            ) -> Self {
                Self {
                    id,
                    version,
                    target_platform,
                    download_type,
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

        #[native_model(id = 9, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
        #[derive(Serialize, Deserialize, Clone, Default)]

        pub struct DatabaseCompatInfo {
            pub umu_installed: bool,
        }

        #[native_model(id = 1, version = 1)]
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
    }

    impl Database {
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
                compat_info: None,
            }
        }
    }
    impl DatabaseAuth {
        pub fn new(
            private: String,
            cert: String,
            client_id: String,
            web_token: Option<String>,
        ) -> Self {
            Self {
                private,
                cert,
                client_id,
                web_token,
            }
        }
    }
}
