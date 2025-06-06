use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Mutex,
};

use log::{error, warn};
use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

#[derive(Serialize, Deserialize, Debug)]
pub struct DropData {
    game_id: String,
    game_version: String,
    pub completed_contexts: Mutex<Vec<usize>>,
    pub base_path: PathBuf,
}

static DROP_DATA_PATH: &str = ".dropdata";

impl DropData {
    pub fn new(game_id: String, game_version: String, base_path: PathBuf) -> Self {
        Self {
            base_path,
            game_id,
            game_version,
            completed_contexts: Mutex::new(Vec::new()),
        }
    }
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

        match serde_binary::from_vec::<DropData>(s, Endian::Little) {
            Ok(manifest) => manifest,
            Err(e) => {
                warn!("{}", e);
                DropData::new(game_id, game_version, base_path)
            }
        }
    }
    pub fn write(&self) {
        let manifest_raw = match serde_binary::to_vec(&self, Endian::Little) {
            Ok(json) => json,
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
    pub fn set_completed_contexts(&self, completed_contexts: &[usize]) {
        *self.completed_contexts.lock().unwrap() = completed_contexts.to_owned();
    }
    pub fn get_completed_contexts(&self) -> Vec<usize> {
        self.completed_contexts.lock().unwrap().clone()
    }
}
