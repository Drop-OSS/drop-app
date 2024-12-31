use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex, RwLockWriteGuard},
};

use directories::BaseDirs;
use log::debug;
use rustbreak::{DeSerError, DeSerializer, PathDatabase};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::AppHandle;
use url::Url;

use crate::{library::push_application_update, process::process_manager::Platform, state::DownloadStatusManager, DB};

#[derive(serde::Serialize, Clone, Deserialize)]
pub struct DatabaseAuth {
    pub private: String,
    pub cert: String,
    pub client_id: String,
}

// Strings are version names for a particular game
#[derive(Serialize, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ApplicationStatus {
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationVersion {
    pub version_index: usize,
    pub version_name: String,
    pub launch_command: String,
    pub setup_command: String,
    pub platform: Platform,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseApplications {
    pub install_dirs: Vec<String>,
    // Guaranteed to exist if the game also exists in the app state map
    pub statuses: HashMap<String, ApplicationStatus>,
    pub versions: HashMap<String, HashMap<String, ApplicationVersion>>,

    #[serde(skip)]
    pub transient_statuses: HashMap<String, ApplicationTransientStatus>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub autostart: bool,
    // ... other settings ...
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            // ... other settings defaults ...
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    #[serde(default)]
    pub settings: Settings,
    pub auth: Option<DatabaseAuth>,
    pub base_url: String,
    pub applications: DatabaseApplications,
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

        debug!("Creating data directory at {:?}", data_root_dir);
        create_dir_all(data_root_dir.clone()).unwrap();
        debug!("Creating games directory");
        create_dir_all(games_base_dir.clone()).unwrap();
        debug!("Creating logs directory");
        create_dir_all(logs_root_dir.clone()).unwrap();

        let exists = fs::exists(db_path.clone()).unwrap();

        match exists {
            true => PathDatabase::load_from_path(db_path).expect("Database loading failed"),
            false => {
                let default = Database {
                    settings: Settings::default(),
                    auth: None,
                    base_url: "".to_string(),
                    applications: DatabaseApplications {
                        install_dirs: vec![games_base_dir.to_str().unwrap().to_string()],
                        statuses: HashMap::new(),
                        transient_statuses: HashMap::new(),
                        versions: HashMap::new(),
                    },
                };
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

#[tauri::command]
pub fn add_download_dir(new_dir: String) -> Result<(), String> {
    // Check the new directory is all good
    let new_dir_path = Path::new(&new_dir);
    if new_dir_path.exists() {
        let metadata = new_dir_path
            .metadata()
            .map_err(|e| format!("Unable to access file or directory: {}", e))?;
        if !metadata.is_dir() {
            return Err("Invalid path: not a directory".to_string());
        }
        let dir_contents = new_dir_path
            .read_dir()
            .map_err(|e| format!("Unable to check directory contents: {}", e))?;
        if dir_contents.count() != 0 {
            return Err("Directory is not empty".to_string());
        }
    } else {
        create_dir_all(new_dir_path)
            .map_err(|e| format!("Unable to create directories to path: {}", e))?;
    }

    // Add it to the dictionary
    let mut lock = DB.borrow_data_mut().unwrap();
    if lock.applications.install_dirs.contains(&new_dir) {
        return Err("Download directory already used".to_string());
    }
    lock.applications.install_dirs.push(new_dir);
    drop(lock);
    DB.save().unwrap();

    Ok(())
}

#[tauri::command]
pub fn delete_download_dir(index: usize) -> Result<(), String> {
    let mut lock = DB.borrow_data_mut().unwrap();
    lock.applications.install_dirs.remove(index);
    drop(lock);
    DB.save().unwrap();

    Ok(())
}

// Will, in future, return disk/remaining size
// Just returns the directories that have been set up
#[tauri::command]
pub fn fetch_download_dir_stats() -> Result<Vec<String>, String> {
    let lock = DB.borrow_data().unwrap();
    let directories = lock.applications.install_dirs.clone();
    drop(lock);

    Ok(directories)
}

pub fn set_application_status<F: FnOnce(&mut RwLockWriteGuard<'_, Database>, &String)>(
    app_handle: &AppHandle,
    id: String,
    setter: F,
) {
    let mut db_handle = DB.borrow_data_mut().unwrap();
    setter(&mut db_handle, &id);
    drop(db_handle);
    DB.save().unwrap();

    let status = DownloadStatusManager::fetch_state(&id);

    push_application_update(app_handle, id, status);
}
