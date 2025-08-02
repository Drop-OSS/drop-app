use std::{
    collections::HashMap, fs::File, io::{self, Read, Write}, path::PathBuf
};

use log::{error, info};
use native_model::{Decode, Encode};

pub type DropData = v1::DropData;

pub static DROP_DATA_PATH: &str = ".dropdata";

pub mod v1 {
    use std::{collections::HashMap, path::PathBuf, sync::Mutex};

    use native_model::native_model;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    #[native_model(id = 9, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
    pub struct DropData {
        pub game_id: String,
        pub game_version: String,
        pub contexts: Mutex<HashMap<String, bool>>,
        pub base_path: PathBuf,
    }

    impl DropData {
        pub fn new(game_id: String, game_version: String, base_path: PathBuf) -> Self {
            Self {
                base_path,
                game_id,
                game_version,
                contexts: Mutex::new(HashMap::new()),
            }
        }
    }
}

impl DropData {
    pub fn generate(game_id: String, game_version: String, base_path: PathBuf) -> Self {
        match DropData::read(&base_path) {
            Ok(v) => v,
            Err(_) => DropData::new(game_id, game_version, base_path),
        }
    }
    pub fn read(base_path: &PathBuf) -> Result<Self, io::Error> {
        let mut file = File::open(base_path.join(DROP_DATA_PATH))?;

        let mut s = Vec::new();
        file.read_to_end(&mut s)?;

        Ok(native_model::rmp_serde_1_3::RmpSerde::decode(s).unwrap())
    }
    pub fn write(&self) {
        let manifest_raw = match native_model::rmp_serde_1_3::RmpSerde::encode(&self) {
            Ok(data) => data,
            Err(_) => return,
        };

        let mut file = match File::create(self.base_path.join(DROP_DATA_PATH)) {
            Ok(file) => file,
            Err(e) => {
                error!("{e}");
                return;
            }
        };

        match file.write_all(&manifest_raw) {
            Ok(()) => {}
            Err(e) => error!("{e}"),
        }
    }
    pub fn set_contexts(&self, completed_contexts: &[(String, bool)]) {
        *self.contexts.lock().unwrap() = completed_contexts.iter().map(|s| (s.0.clone(), s.1)).collect();
    }
    pub fn set_context(&self, context: String, state: bool) {
        self.contexts.lock().unwrap().entry(context).insert_entry(state);
    }
    pub fn get_completed_contexts(&self) -> Vec<String> {
        self.contexts
            .lock()
            .unwrap()
            .iter()
            .filter_map(|x| if *x.1 { Some(x.0.clone()) } else { None })
            .collect()
    }
    pub fn get_contexts(&self) -> HashMap<String, bool> {
        self.contexts.lock().unwrap().clone()
    }
}
