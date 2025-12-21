use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use keyring::Entry;
use log::info;

use crate::interface::{DatabaseInterface};

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

pub(crate) static KEY_IV: LazyLock<([u8; 16], [u8; 16])> = LazyLock::new(|| {
    let entry = Entry::new("drop", "database_key").expect("failed to open keyring");
    let mut key = entry.get_secret().unwrap_or_else(|_| {
        let mut buffer = [0u8; 32];
        rand::fill(&mut buffer);
        entry.set_secret(&buffer).expect("failed to save key");
        info!("created new database key");
        buffer.to_vec()
    });
    let new = key.split_off(16);
    (new.try_into().expect("failed to extract key"), key.try_into().expect("failed to extract iv"))
});