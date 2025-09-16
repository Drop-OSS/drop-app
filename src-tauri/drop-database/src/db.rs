use std::{
    fs::{self, create_dir_all},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, LazyLock, RwLockReadGuard, RwLockWriteGuard},
};

use chrono::Utc;
use drop_consts::DATA_ROOT_DIR;
use log::{debug, error, info, warn};
use rustbreak::{DeSerError, DeSerializer, PathDatabase, RustbreakError};
use serde::{Serialize, de::DeserializeOwned};

use crate::DB;

use super::models::data::Database;

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
            .map_err(|e| rustbreak::error::DeSerError::Internal(e.to_string()))?;
        let (val, _version) =
            native_model::decode(buf).map_err(|e| DeSerError::Internal(e.to_string()))?;
        Ok(val)
    }
}

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, DropDatabaseSerializer>;

pub trait DatabaseImpls {
    fn set_up_database() -> DatabaseInterface;
}
impl DatabaseImpls for DatabaseInterface {
    fn set_up_database() -> DatabaseInterface {
        let db_path = DATA_ROOT_DIR.join("drop.db");
        let games_base_dir = DATA_ROOT_DIR.join("games");
        let logs_root_dir = DATA_ROOT_DIR.join("logs");
        let cache_dir = DATA_ROOT_DIR.join("cache");
        let pfx_dir = DATA_ROOT_DIR.join("pfx");

        debug!("creating data directory at {DATA_ROOT_DIR:?}");
        create_dir_all(DATA_ROOT_DIR.as_path()).unwrap();
        create_dir_all(&games_base_dir).unwrap();
        create_dir_all(&logs_root_dir).unwrap();
        create_dir_all(&cache_dir).unwrap();
        create_dir_all(&pfx_dir).unwrap();

        let exists = fs::exists(db_path.clone()).unwrap();

        if exists {
            match PathDatabase::load_from_path(db_path.clone()) {
                Ok(db) => db,
                Err(e) => handle_invalid_database(e, db_path, games_base_dir, cache_dir),
            }
        } else {
            let default = Database::new(games_base_dir, None);
            debug!(
                "Creating database at path {}",
                db_path.as_os_str().to_str().unwrap()
            );
            PathDatabase::create_at_path(db_path, default).expect("Database could not be created")
        }
    }
}

// TODO: Make the error relelvant rather than just assume that it's a Deserialize error
fn handle_invalid_database(
    _e: RustbreakError,
    db_path: PathBuf,
    games_base_dir: PathBuf,
    cache_dir: PathBuf,
) -> rustbreak::Database<Database, rustbreak::backend::PathBackend, DropDatabaseSerializer> {
    warn!("{_e}");
    let new_path = {
        let time = Utc::now().timestamp();
        let mut base = db_path.clone();
        base.set_file_name(format!("drop.db.backup-{time}"));
        base
    };
    info!("old database stored at: {}", new_path.to_string_lossy());
    fs::rename(&db_path, &new_path).unwrap();

    let db = Database::new(
        games_base_dir.into_os_string().into_string().unwrap(),
        Some(new_path),
    );

    PathDatabase::create_at_path(db_path, db).expect("Database could not be created")
}

// To automatically save the database upon drop
pub struct DBRead<'a>(pub(crate) RwLockReadGuard<'a, Database>);
pub struct DBWrite<'a>(pub(crate) ManuallyDrop<RwLockWriteGuard<'a, Database>>);
impl<'a> Deref for DBWrite<'a> {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> DerefMut for DBWrite<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<'a> Deref for DBRead<'a> {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for DBWrite<'_> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.0);
        }

        match DB.save() {
            Ok(()) => {}
            Err(e) => {
                error!("database failed to save with error {e}");
                panic!("database failed to save with error {e}")
            }
        }
    }
}
