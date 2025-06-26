use std::{fs::File, io::{Read, Write}, path::PathBuf};

use log::{debug, error, info, warn};
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
        pub contexts: Mutex<Vec<(String, bool)>>,
        pub base_path: PathBuf,
    }

    impl DropData {
        pub fn new(game_id: String, game_version: String, base_path: PathBuf) -> Self {
            Self {
                base_path,
                game_id,
                game_version,
                contexts: Mutex::new(Vec::new()),
            }
        }
    }
}

impl DropData {
    pub fn generate(game_id: String, game_version: String, base_path: PathBuf) -> Self {
        let mut file = match File::open(base_path.join(DROP_DATA_PATH)) {
            Ok(file) => file,
            Err(_) => {
                debug!("Generating new dropdata for game {}", game_id);
                return DropData::new(game_id, game_version, base_path)
            },
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
    pub fn set_contexts(&self, completed_contexts: &[(String, bool)]) {
        *self.contexts.lock().unwrap() = completed_contexts.to_owned();
    }
    pub fn get_completed_contexts(&self) -> Vec<String> {
        self.contexts.lock().unwrap().iter().filter_map(|x| { if x.1 { Some(x.0.clone()) } else { None } }).collect()
    }
    pub fn get_contexts(&self) -> Vec<(String, bool)> {
        info!("Any contexts which are complete? {}", self.contexts.lock().unwrap().iter().any(|x| x.1));
        self.contexts.lock().unwrap().clone()
    }
}
