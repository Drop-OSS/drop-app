use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    hash::Hash,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex, RwLockReadGuard, RwLockWriteGuard},
};

use chrono::Utc;
use directories::BaseDirs;
use log::{debug, error, info};
use rustbreak::{DeSerError, DeSerializer, PathDatabase, RustbreakError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use tauri::AppHandle;
use url::Url;

use crate::{
    database::settings::Settings,
    download_manager::downloadable_metadata::DownloadableMetadata,
    games::{library::push_game_update, state::GameStatusManager},
    process::process_manager::Platform,
    DB,
};

#[derive(serde::Serialize, Clone, Deserialize)]
pub struct DatabaseAuth {
    pub private: String,
    pub cert: String,
    pub client_id: String,
}

// Strings are version names for a particular game
#[derive(Serialize, Clone, Deserialize)]
#[serde(tag = "type")]
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
#[derive(Clone, Serialize)]
pub enum ApplicationTransientStatus {
    Downloading { version_name: String },
    Uninstalling {},
    Updating { version_name: String },
    Running {},
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    pub game_id: String,
    pub version_name: String,

    pub platform: Platform,

    pub launch_command: String,
    pub launch_args: Vec<String>,

    pub setup_command: String,
    pub setup_args: Vec<String>,

    pub only_setup: bool,

    pub version_index: usize,
    pub delta: bool,

    pub umu_id_override: Option<String>,
}

#[serde_as]
#[derive(Serialize, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseApplications {
    pub install_dirs: Vec<PathBuf>,
    // Guaranteed to exist if the game also exists in the app state map
    pub game_statuses: HashMap<String, GameDownloadStatus>,
    pub game_versions: HashMap<String, HashMap<String, GameVersion>>,
    pub installed_game_version: HashMap<String, DownloadableMetadata>,

    #[serde(skip)]
    pub transient_statuses: HashMap<DownloadableMetadata, ApplicationTransientStatus>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Database {
    #[serde(default)]
    pub settings: Settings,
    pub auth: Option<DatabaseAuth>,
    pub base_url: String,
    pub applications: DatabaseApplications,
    pub prev_database: Option<PathBuf>,
    pub cache_dir: PathBuf
}
impl Database {
    fn new<T: Into<PathBuf>>(games_base_dir: T, prev_database: Option<PathBuf>, cache_dir: PathBuf) -> Self {
        Self {
            applications: DatabaseApplications {
                install_dirs: vec![games_base_dir.into()],
                game_statuses: HashMap::new(),
                game_versions: HashMap::new(),
                installed_game_version: HashMap::new(),
                transient_statuses: HashMap::new(),
            },
            prev_database,
            base_url: "".to_owned(),
            auth: None,
            settings: Settings::default(),
            cache_dir,
        }
    }
}
pub static DATA_ROOT_DIR: LazyLock<Mutex<PathBuf>> =
    LazyLock::new(|| Mutex::new(BaseDirs::new().unwrap().data_dir().join("drop")));

// Custom JSON serializer to support everything we need
#[derive(Debug, Default, Clone)]
pub struct DropDatabaseSerializer;

impl<T: Serialize + DeserializeOwned> DeSerializer<T> for DropDatabaseSerializer {
    fn serialize(&self, val: &T) -> rustbreak::error::DeSerResult<Vec<u8>> {
        serde_json::to_vec(val).map_err(|e| DeSerError::Internal(e.to_string()))
    }

    fn deserialize<R: std::io::Read>(&self, s: R) -> rustbreak::error::DeSerResult<T> {
        serde_json::from_reader(s).map_err(|e| DeSerError::Internal(e.to_string()))
    }
}

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, DropDatabaseSerializer>;

pub trait DatabaseImpls {
    fn set_up_database() -> DatabaseInterface;
    fn database_is_set_up(&self) -> bool;
    fn fetch_base_url(&self) -> Url;
}
impl DatabaseImpls for DatabaseInterface {
    fn set_up_database() -> DatabaseInterface {
        let data_root_dir = DATA_ROOT_DIR.lock().unwrap();
        let db_path = data_root_dir.join("drop.db");
        let games_base_dir = data_root_dir.join("games");
        let logs_root_dir = data_root_dir.join("logs");
        let cache_dir = data_root_dir.join("cache/");

        debug!("creating data directory at {:?}", data_root_dir);
        create_dir_all(data_root_dir.clone()).unwrap();
        create_dir_all(&games_base_dir).unwrap();
        create_dir_all(&logs_root_dir).unwrap();
        create_dir_all(&cache_dir).unwrap();

        let exists = fs::exists(db_path.clone()).unwrap();

        match exists {
            true => match PathDatabase::load_from_path(db_path.clone()) {
                Ok(db) => db,
                Err(e) => handle_invalid_database(e, db_path, games_base_dir, cache_dir),
            },
            false => {
                let default = Database::new(games_base_dir, None, cache_dir);
                debug!(
                    "Creating database at path {}",
                    db_path.as_os_str().to_str().unwrap()
                );
                PathDatabase::create_at_path(db_path, default)
                    .expect("Database could not be created")
            }
        }
    }

    fn database_is_set_up(&self) -> bool {
        !self.borrow_data().unwrap().base_url.is_empty()
    }

    fn fetch_base_url(&self) -> Url {
        let handle = self.borrow_data().unwrap();
        Url::parse(&handle.base_url).unwrap()
    }
}

pub fn set_game_status<F: FnOnce(&mut RwLockWriteGuard<'_, Database>, &DownloadableMetadata)>(
    app_handle: &AppHandle,
    meta: DownloadableMetadata,
    setter: F,
) {
    let mut db_handle = borrow_db_mut_checked();
    setter(&mut db_handle, &meta);
    drop(db_handle);
    save_db();

    let status = GameStatusManager::fetch_state(&meta.id);

    push_game_update(app_handle, &meta.id, status);
}
// TODO: Make the error relelvant rather than just assume that it's a Deserialize error
fn handle_invalid_database(
    _e: RustbreakError,
    db_path: PathBuf,
    games_base_dir: PathBuf,
    cache_dir: PathBuf
) -> rustbreak::Database<Database, rustbreak::backend::PathBackend, DropDatabaseSerializer> {
    let new_path = {
        let time = Utc::now().timestamp();
        let mut base = db_path.clone();
        base.set_file_name(format!("drop.db.backup-{}", time));
        base
    };
    info!(
        "old database stored at: {}",
        new_path.to_string_lossy().to_string()
    );
    fs::rename(&db_path, &new_path).unwrap();

    let db = Database::new(
        games_base_dir.into_os_string().into_string().unwrap(),
        Some(new_path),
        cache_dir
    );

    PathDatabase::create_at_path(db_path, db).expect("Database could not be created")
}

pub fn borrow_db_checked<'a>() -> RwLockReadGuard<'a, Database> {
    match DB.borrow_data() {
        Ok(data) => data,
        Err(e) => {
            error!("database borrow failed with error {}", e);
            panic!("database borrow failed with error {}", e);
        }
    }
}

pub fn borrow_db_mut_checked<'a>() -> RwLockWriteGuard<'a, Database> {
    match DB.borrow_data_mut() {
        Ok(data) => data,
        Err(e) => {
            error!("database borrow mut failed with error {}", e);
            panic!("database borrow mut failed with error {}", e);
        }
    }
}

pub fn save_db() {
    match DB.save() {
        Ok(_) => {}
        Err(e) => {
            error!("database failed to save with error {}", e);
            panic!("database failed to save with error {}", e)
        }
    }
}
