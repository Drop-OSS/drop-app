use std::{fs::File, io::{Read, Write}, path::PathBuf};

use log::{error, warn};
use native_model::{Decode, Encode};

pub type DropData = v1::DropData;

static DROP_DATA_PATH: &str = ".dropdata";


pub mod v1 {
    use std::{path::PathBuf, sync::Mutex};

    use native_model::native_model;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    #[native_model(id = 9, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
    pub struct DropData {
        game_id: String,
        game_version: String,
        pub completed_contexts: Mutex<Vec<String>>,
        pub base_path: PathBuf,
    }

    impl DropData {
        pub fn new(game_id: String, game_version: String, base_path: PathBuf) -> Self {
            Self {
                base_path,
                game_id,
                game_version,
                completed_contexts: Mutex::new(Vec::new()),
            }
        }
    }
}

impl DropData {
    pub fn generate(game_id: String, game_version: String, base_path: PathBuf) -> Self {
        let mut file = match File::open(base_path.join(DROP_DATA_PATH)) {
            Ok(file) => file,
            Err(_) => return DropData::new(game_id, game_version, base_path),
        };

        let mut s = Vec::new();
        match file.read_to_end(&mut s) {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e);
                return DropData::new(game_id, game_version, base_path);
            }
        };

        match native_model::rmp_serde_1_3::RmpSerde::decode(s) {
            Ok(manifest) => manifest,
            Err(e) => {
                warn!("{}", e);
                DropData::new(game_id, game_version, base_path)
            }
        }
    }
    pub fn write(&self) {
        let manifest_raw = match native_model::rmp_serde_1_3::RmpSerde::encode(&self) {
            Ok(data) => data,
            Err(_) => return,
        };

        let mut file = match File::create(self.base_path.join(DROP_DATA_PATH)) {
            Ok(file) => file,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        match file.write_all(&manifest_raw) {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        };
    }
    pub fn set_completed_contexts(&self, completed_contexts: &[String]) {
        *self.completed_contexts.lock().unwrap() = completed_contexts.to_owned();
    }
    pub fn get_completed_contexts(&self) -> Vec<String> {
        self.completed_contexts.lock().unwrap().clone()
    }
}
