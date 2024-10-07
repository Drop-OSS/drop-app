use std::fs::create_dir_all;

use directories::BaseDirs;
use rustbreak::{deser::Bincode, FileDatabase, PathDatabase};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseCerts {
    pub private: String,
    pub public: String,
    pub cert: String,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Database {
    pub certs: Option<DatabaseCerts>,
}

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, Bincode>;

pub fn setup() -> DatabaseInterface {
    let db_path = BaseDirs::new().unwrap().data_dir().join("drop");
    let default = Database {
        certs: None
    };
    let db = PathDatabase::<Database, Bincode>::create_at_path(db_path, default).unwrap();

    db.save().unwrap();

    return db;
}
