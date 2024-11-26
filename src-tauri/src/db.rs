use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use directories::BaseDirs;
use log::{debug, info};
use rustbreak::{deser::Bincode, PathDatabase};
use rustix::path::Arg;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::DB;

#[derive(serde::Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseAuth {
    pub private: String,
    pub cert: String,
    pub client_id: String,
}

#[derive(Serialize, Clone, Deserialize)]
pub enum DatabaseGameStatus {
    Remote,
    Downloading,
    Installed,
    Updating,
    Uninstalling,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseGames {
    pub install_dirs: Vec<String>,
    // Guaranteed to exist if the game also exists in the app state map
    pub games_statuses: HashMap<String, DatabaseGameStatus>,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub auth: Option<DatabaseAuth>,
    pub base_url: String,
    pub games: DatabaseGames,
}
pub static DATA_ROOT_DIR: LazyLock<Mutex<PathBuf>> =
    LazyLock::new(|| Mutex::new(BaseDirs::new().unwrap().data_dir().join("drop")));

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, Bincode>;

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

        debug!("Creating data directory at {:?}", data_root_dir);
        create_dir_all(data_root_dir.clone()).unwrap();
        debug!("Creating games directory");
        create_dir_all(games_base_dir.clone()).unwrap();

        #[allow(clippy::let_and_return)]
        let exists = fs::exists(db_path.clone()).unwrap();
        let db = match exists {
            true => PathDatabase::load_from_path(db_path).expect("Database loading failed"),
            false => {
                let default = Database {
                    auth: None,
                    base_url: "".to_string(),
                    games: DatabaseGames {
                        install_dirs: vec![games_base_dir.to_str().unwrap().to_string()],
                        games_statuses: HashMap::new(),
                    },
                };
                debug!("Creating database at path {}", db_path.as_str().unwrap());
                PathDatabase::create_at_path(db_path, default)
                    .expect("Database could not be created")
            }
        };

        db
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
    if lock.games.install_dirs.contains(&new_dir) {
        return Err("Download directory already used".to_string());
    }
    lock.games.install_dirs.push(new_dir);
    drop(lock);
    DB.save().unwrap();

    Ok(())
}

#[tauri::command]
pub fn delete_download_dir(index: usize) -> Result<(), String> {
    let mut lock = DB.borrow_data_mut().unwrap();
    lock.games.install_dirs.remove(index);
    drop(lock);
    DB.save().unwrap();

    Ok(())
}

// Will, in future, return disk/remaining size
// Just returns the directories that have been set up
#[tauri::command]
pub fn fetch_download_dir_stats() -> Result<Vec<String>, String> {
    let lock = DB.borrow_data().unwrap();
    let directories = lock.games.install_dirs.clone();
    drop(lock);

    Ok(directories)
}
