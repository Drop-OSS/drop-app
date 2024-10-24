use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    path::PathBuf,
    sync::LazyLock,
};

use directories::BaseDirs;
use rustbreak::{deser::Bincode, PathDatabase};
use serde::{Deserialize, Serialize};
use url::Url;

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
    pub games_base_dir: String,
    pub games_statuses: HashMap<String, DatabaseGameStatus>,
}

#[derive(Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub auth: Option<DatabaseAuth>,
    pub base_url: String,
    pub games: DatabaseGames,
}
pub static DATA_ROOT_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| BaseDirs::new().unwrap().data_dir().join("drop"));

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, Bincode>;

pub trait DatabaseImpls {
    fn set_up_database() -> DatabaseInterface;
    fn database_is_set_up(&self) -> bool;
    fn fetch_base_url(&self) -> Url;
}
impl DatabaseImpls for DatabaseInterface {
    fn set_up_database() -> DatabaseInterface {
        let db_path = DATA_ROOT_DIR.join("drop.db");
        let games_base_dir = DATA_ROOT_DIR.join("games");

        create_dir_all(DATA_ROOT_DIR.clone()).unwrap();
        create_dir_all(games_base_dir.clone()).unwrap();

        let default = Database {
            auth: None,
            base_url: "".to_string(),
            games: DatabaseGames {
                games_base_dir: games_base_dir.to_str().unwrap().to_string(),
                games_statuses: HashMap::new(),
            },
        };
        #[allow(clippy::let_and_return)]
        let db = match fs::exists(db_path.clone()).unwrap() {
            true => PathDatabase::load_from_path(db_path).expect("Database loading failed"),
            false => PathDatabase::create_at_path(db_path, default).unwrap(),
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