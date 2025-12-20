use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use aes::cipher::{KeyIvInit as _, StreamCipher};
use keyring::Entry;
use log::info;
use rustbreak::{DeSerError, DeSerializer};
use serde::{Serialize, de::DeserializeOwned};

use crate::interface::{DatabaseImpls, DatabaseInterface};

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(DatabaseInterface::set_up_database);

#[cfg(not(debug_assertions))]
static DATA_ROOT_PREFIX: &str = "drop";
#[cfg(debug_assertions)]
static DATA_ROOT_PREFIX: &str = "drop-debug";

type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes128>;

pub static DATA_ROOT_DIR: LazyLock<Arc<PathBuf>> = LazyLock::new(|| {
    Arc::new(
        dirs::data_dir()
            .expect("Failed to get data dir")
            .join(DATA_ROOT_PREFIX),
    )
});

static KEY: LazyLock<[u8; 16]> = LazyLock::new(|| {
    let entry = Entry::new("drop", "key").expect("failed to open keyring");
    let key = entry.get_secret().unwrap_or_else(|_| {
        let mut buffer = [0u8; 16];
        rand::fill(&mut buffer);
        entry.set_secret(&buffer).expect("failed to save key");
        info!("created new database key");
        buffer.to_vec()
    });
    key.try_into().expect("failed to extract appropriate key")
});

static IV: LazyLock<[u8; 16]> = LazyLock::new(|| {
    let entry = Entry::new("drop", "iv").expect("failed to open keyring");
    let iv = entry.get_secret().unwrap_or_else(|_| {
        let mut buffer = [0u8; 16];
        rand::fill(&mut buffer);
        entry.set_secret(&buffer).expect("failed to save iv");
        info!("created new database iv");
        buffer.to_vec()
    });
    iv.try_into().expect("failed to extract appropriate iv")
});

// Custom JSON serializer to support everything we need
#[derive(Debug, Default, Clone)]
pub struct DropDatabaseSerializer;

impl<T: native_model::Model + Serialize + DeserializeOwned> DeSerializer<T>
    for DropDatabaseSerializer
{
    fn serialize(&self, val: &T) -> rustbreak::error::DeSerResult<Vec<u8>> {
        let mut data =
            native_model::encode(val).map_err(|e| DeSerError::Internal(e.to_string()))?;
        let key: [u8; 16] = *KEY;
        let iv: [u8; 16] = *IV;
        let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());
        cipher.apply_keystream(&mut data);
        Ok(data)
    }

    fn deserialize<R: std::io::Read>(&self, mut s: R) -> rustbreak::error::DeSerResult<T> {
        let mut buf = Vec::new();
        s.read_to_end(&mut buf)
            .map_err(|e| rustbreak::error::DeSerError::Other(e.into()))?;
        let key: [u8; 16] = *KEY;
        let iv: [u8; 16] = *IV;
        let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());
        cipher.apply_keystream(&mut buf);
        let (val, _version) =
            native_model::decode(buf).map_err(|e| DeSerError::Internal(e.to_string()))?;
        Ok(val)
    }
}
