use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    sync::LazyLock,
};

use directories::BaseDirs;
use rustbreak::{deser::Bincode, PathDatabase};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::DB;

#[derive(serde::Serialize, Clone, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct DatabaseAuth {
    pub private: String,
    pub cert: String,
    pub client_id: String,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct DatabaseApps {
    pub apps_base_dir: String,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Database {
    pub auth: Option<DatabaseAuth>,
    pub base_url: String,
    pub downloads: DatabaseApps,
}

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, Bincode>;

pub static DATA_ROOT_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| BaseDirs::new().unwrap().data_dir().join("drop"));

pub fn setup() -> DatabaseInterface {
    let db_path = DATA_ROOT_DIR.join("drop.db");
    let apps_base_dir = DATA_ROOT_DIR.join("apps");

    create_dir_all(DATA_ROOT_DIR.clone()).unwrap();
    create_dir_all(apps_base_dir.clone()).unwrap();

    let default = Database {
        auth: None,
        base_url: "".to_string(),
        downloads: DatabaseApps {
            apps_base_dir: apps_base_dir.to_str().unwrap().to_string(),
        },
    };
    #[allow(clippy::let_and_return)]
    let db = match fs::exists(db_path.clone()).unwrap() {
        true => PathDatabase::load_from_path(db_path).expect("Database loading failed"),
        false => PathDatabase::create_at_path(db_path, default).unwrap(),
    };

    db
}

pub fn is_set_up() -> bool {
    return !DB.borrow_data().unwrap().base_url.is_empty();
}

pub fn fetch_base_url() -> Url {
    let handle = DB.borrow_data().unwrap();
    Url::parse(&handle.base_url).unwrap()
}
