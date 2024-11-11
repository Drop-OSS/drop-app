use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use super::{download_agent::GameDownloadAgent, download_thread_control_flag::DownloadThreadControlFlag};

pub struct DownloadManager {
    download_agent_registry: HashMap<String, Arc<Mutex<GameDownloadAgent>>>,
    download_queue: Vec<String>,

    current_thread: Option<JoinHandle<()>>,
    current_game_id: Option<String>, // Should be the only game download agent in the map with the "Go" flag
}

impl DownloadManager {
    pub fn new() -> Self {
        return Self {
            download_agent_registry: HashMap::new(),
            download_queue: Vec::new(),
            current_thread: None,
            current_game_id: None,
        };
    }

    pub fn queue_game(&mut self, game_id: String, version_name: String) {
        let existing_da = self.download_agent_registry.get(&game_id);

        if let Some(da_mutex) = existing_da {
            let da = da_mutex.lock().unwrap();
            if da.version == version_name {
                return; // We're already queued
            }

            da.control_flag.set(DownloadThreadControlFlag::Stop);
            
        }
    }
}
