use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use database::{models::data::UserConfiguration, platform::Platform};
use log::error;
use utils::lock;

pub type DropData = v1::DropData;

pub static DROPDATA_PATH: &str = ".dropdata";

pub mod v1 {
    use std::{collections::HashMap, path::PathBuf, sync::Mutex};

    use database::{models::data::UserConfiguration, platform::Platform};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DropData {
        pub game_id: String,
        pub game_version: String,
        pub target_platform: Platform,
        #[serde(default)]
        pub configuration: UserConfiguration,
        pub contexts: Mutex<HashMap<String, bool>>,
        pub base_path: PathBuf,
        pub previously_installed_version: Option<String>,
    }

    impl DropData {
        pub fn new(
            game_id: String,
            game_version: String,
            target_platform: Platform,
            base_path: PathBuf,
            configuration: UserConfiguration,
            previously_installed_version: Option<String>,
        ) -> Self {
            Self {
                base_path,
                game_id,
                game_version,
                target_platform,
                contexts: Mutex::new(HashMap::new()),
                configuration,
                previously_installed_version,
            }
        }
    }
}

impl DropData {
    pub fn generate(
        game_id: String,
        game_version: String,
        target_platform: Platform,
        base_path: PathBuf,
        configuration: UserConfiguration,
    ) -> Self {
        match DropData::read(&base_path) {
            Ok(v) => {
                if v.game_id != game_id || v.game_version != game_version {
                    return DropData::new(
                        game_id,
                        game_version,
                        target_platform,
                        base_path,
                        configuration,
                        Some(v.game_version),
                    );
                }
                v
            }
            Err(_) => DropData::new(
                game_id,
                game_version,
                target_platform,
                base_path,
                configuration,
                None,
            ),
        }
    }
    pub fn read(base_path: &Path) -> Result<Self, io::Error> {
        let mut file = File::open(base_path.join(DROPDATA_PATH))?;

        let mut s = Vec::new();
        file.read_to_end(&mut s)?;

        pot::from_slice(&s).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to decode drop data: {e}"),
            )
        })
    }
    pub fn write(&self) {
        let manifest_raw = match pot::to_vec(&self) {
            Ok(data) => data,
            Err(_) => return,
        };

        let mut file = match File::create(self.base_path.join(DROPDATA_PATH)) {
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
        *lock!(self.contexts) = completed_contexts
            .iter()
            .map(|s| (s.0.clone(), s.1))
            .collect();
    }
    pub fn set_context(&self, context: String, state: bool) {
        lock!(self.contexts).entry(context).insert_entry(state);
    }
    pub fn get_contexts(&self) -> HashMap<String, bool> {
        lock!(self.contexts).clone()
    }
}
