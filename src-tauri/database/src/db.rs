use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use rustbreak::{DeSerError, DeSerializer};
use serde::{Serialize, de::DeserializeOwned};

use crate::interface::{DatabaseImpls, DatabaseInterface};

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(DatabaseInterface::set_up_database);

#[cfg(not(debug_assertions))]
static DATA_ROOT_PREFIX: &str = "drop";
#[cfg(debug_assertions)]
static DATA_ROOT_PREFIX: &str = "drop-debug";

pub static DATA_ROOT_DIR: LazyLock<Arc<PathBuf>> = LazyLock::new(|| {
    Arc::new(
        dirs::data_dir()
            .expect("Failed to get data dir")
            .join(DATA_ROOT_PREFIX),
    )
});

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
            native_model::decode(buf).map_err(|e| DeSerError::Internal(e.to_string()))?;
        Ok(val)
    }
}
