use std::{default, fs::File, io::{Read, Write}, path::{Path, PathBuf}, sync::Mutex};

use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredManifest {
    game_id: String,
    game_version: String,
    pub completed_contexts: Mutex<Vec<usize>>,
    base_path: PathBuf
}

impl StoredManifest {
    pub fn new(game_id: String, game_version: String, base_path: PathBuf) -> Self {
        Self {
            base_path,
            game_id,
            game_version,
            completed_contexts: Mutex::new(Vec::new()),
        }
    }
    pub fn generate(game_id: String, game_version: String, base_path: PathBuf) -> Self {
        let mut file = match File::open(base_path.join("manifest.json")) {
            Ok(file) => file,
            Err(_) => return StoredManifest::new(game_id, game_version, base_path),
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Ok(_) => {},
            Err(e) => { error!("{}", e); return StoredManifest::new(game_id, game_version, base_path) },
        };

        info!("Contexts string: {}", s);
        let manifest = match serde_json::from_str::<StoredManifest>(&s) {
            Ok(manifest) => manifest,
            Err(e) => { error!("{}", e); StoredManifest::new(game_id, game_version, base_path) },
        };
        info!("Completed manifest: {:?}", manifest);

        return manifest;
    }
    pub fn write(&self) {
        let manifest_json = match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(_) => return,
        };

        let mut file = match File::create(self.base_path.join("manifest.json")) {
            Ok(file) => file,
            Err(e) => { error!("{}", e); return; },
        };

        match file.write_all(manifest_json.as_bytes()) {
            Ok(_) => {},
            Err(e) => error!("{}", e),
        };
    }
    pub fn set_completed_contexts(&self, completed_contexts: &Mutex<Vec<usize>>) {
        *self.completed_contexts.lock().unwrap() = completed_contexts.lock().unwrap().clone();
    }
    pub fn get_completed_contexts(&self) -> Vec<usize> {
        self.completed_contexts.lock().unwrap().clone()
    }
}