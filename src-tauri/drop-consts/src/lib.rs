use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

#[cfg(not(debug_assertions))]
static DATA_ROOT_PREFIX: &'static str = "drop";
#[cfg(debug_assertions)]
static DATA_ROOT_PREFIX: &str = "drop-debug";

pub static DATA_ROOT_DIR: LazyLock<&'static PathBuf> =
    LazyLock::new(|| Box::leak(Box::new(dirs::data_dir().unwrap().join(DATA_ROOT_PREFIX))));

pub static CACHE_DIR: LazyLock<&'static PathBuf> =
    LazyLock::new(|| Box::leak(Box::new(DATA_ROOT_DIR.join("cache"))));
