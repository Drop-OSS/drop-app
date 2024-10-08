use std::fs;

use directories::BaseDirs;
use rustbreak::{deser::Bincode, PathDatabase};
use serde::Deserialize;

use crate::DB;

#[derive(serde::Serialize, Clone, Deserialize)]
pub struct DatabaseCerts {
    pub private: String,
    pub public: String,
    pub cert: String,
}

#[derive(serde::Serialize, Clone, Deserialize)]
pub struct Database {
    pub certs: Option<DatabaseCerts>,
    pub base_url: String,
}

pub type DatabaseInterface = rustbreak::Database<Database, rustbreak::backend::PathBackend, Bincode>;

pub fn setup() -> DatabaseInterface {
    let db_path = BaseDirs::new().unwrap().data_dir().join("drop");
    let default = Database {
        certs: None,
        base_url: "".to_string(),
    };
    let db = match fs::exists(db_path.clone()).unwrap() {
        true => PathDatabase::load_from_path(db_path).expect("Database loading failed"),
        false => PathDatabase::create_at_path(db_path, default).unwrap(),
    };

    return db;
}

pub fn is_set_up() -> bool {
    return !DB.borrow_data().unwrap().base_url.is_empty();
}
