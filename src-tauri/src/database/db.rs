use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    sync::{LazyLock, Mutex, RwLockReadGuard, RwLockWriteGuard},
};

use chrono::Utc;
use directories::BaseDirs;
use log::{debug, error, info};
use rustbreak::{DeSerError, DeSerializer, PathDatabase, RustbreakError};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use crate::DB;

use super::models::data::Database;

pub static DATA_ROOT_DIR: LazyLock<Mutex<PathBuf>> =
    LazyLock::new(|| Mutex::new(BaseDirs::new().unwrap().data_dir().join("drop")));

// Custom JSON serializer to support everything we need
#[derive(Debug, Default, Clone)]
pub struct DropDatabaseSerializer;

impl<T: native_model::Model + Serialize + DeserializeOwned> DeSerializer<T>
    for DropDatabaseSerializer
{
    fn serialize(&self, val: &T) -> rustbreak::error::DeSerResult<Vec<u8>> {
        native_model::encode(val).map_err(|e| DeSerError::Internal(e.to_string()))
    }

    fn deserialize<R: std::io::Read>(&self, mut s: R) -> rustbreak::error::DeSerResult<T> {
        let mut buf = Vec::new();
        s.read_to_end(&mut buf)
            .map_err(|e| rustbreak::error::DeSerError::Other(e.into()))?;
        let (val, _version) =
            native_model::decode::<T>(buf).map_err(|e| DeSerError::Internal(e.to_string()))?;
        Ok(val)
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
        let cache_dir = data_root_dir.join("cache");
        let pfx_dir = data_root_dir.join("pfx");

        debug!("creating data directory at {:?}", data_root_dir);
        create_dir_all(data_root_dir.clone()).unwrap();
        create_dir_all(&games_base_dir).unwrap();
        create_dir_all(&logs_root_dir).unwrap();
        create_dir_all(&cache_dir).unwrap();
        create_dir_all(&pfx_dir).unwrap();

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

// TODO: Make the error relelvant rather than just assume that it's a Deserialize error
fn handle_invalid_database(
    _e: RustbreakError,
    db_path: PathBuf,
    games_base_dir: PathBuf,
    cache_dir: PathBuf,
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
        cache_dir,
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
