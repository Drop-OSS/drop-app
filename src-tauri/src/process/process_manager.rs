use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Child, Command},
    sync::LazyLock,
};

use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    db::{GameStatus, DATA_ROOT_DIR},
    DB,
};

pub struct ProcessManager {
    current_platform: Platform,
    log_output_dir: PathBuf,
    processes: HashMap<String, Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let root_dir_lock = DATA_ROOT_DIR.lock().unwrap();
        let log_output_dir = root_dir_lock.join("logs");
        drop(root_dir_lock);

        ProcessManager {
            current_platform: if cfg!(windows) {
                Platform::Windows
            } else {
                Platform::Linux
            },

            processes: HashMap::new(),
            log_output_dir,
        }
    }

    fn process_command(&self, raw_command: String) -> (String, Vec<String>) {
        let command_components = raw_command.split(" ").collect::<Vec<&str>>();
        let root = match self.current_platform {
            Platform::Windows => command_components[0].to_string(),
            Platform::Linux => {
                let mut root = command_components[0].to_string();
                if !root.starts_with("./") {
                    root = format!("{}{}", "./", root);
                }
                root
            }
        };
        let args = command_components[1..]
            .into_iter()
            .map(|v| v.to_string())
            .collect();
        (root, args)
    }

    pub fn valid_platform(&self, platform: &Platform) -> Result<bool, String> {
        let current = &self.current_platform;
        let valid_platforms = PROCESS_COMPATABILITY_MATRIX
            .get(current)
            .ok_or("Incomplete platform compatability matrix.")?;

        Ok(valid_platforms.contains(platform))
    }

    pub fn launch_game(&mut self, game_id: String) -> Result<(), String> {
        if self.processes.contains_key(&game_id) {
            return Err("Game or setup is already running.".to_owned());
        }

        let db_lock = DB.borrow_data().unwrap();
        let game_status = db_lock
            .games
            .statuses
            .get(&game_id)
            .ok_or("Game not installed")?;

        let GameStatus::Installed {
            version_name,
            install_dir,
        } = game_status
        else {
            return Err("Game not installed.".to_owned());
        };

        let game_version = db_lock
            .games
            .versions
            .get(&game_id)
            .ok_or("Invalid game ID".to_owned())?
            .get(version_name)
            .ok_or("Invalid version name".to_owned())?;

        let (command, args) = self.process_command(game_version.launch_command.clone());

        info!("launching process {} in {}", command, install_dir);

        let current_time = chrono::offset::Local::now();
        let mut log_file = OpenOptions::new()
            .append(true)
            .read(true)
            .create(true)
            .open(self.log_output_dir.join(format!(
                "{}-{}.log",
                game_id,
                current_time.timestamp()
            )))
            .map_err(|v| v.to_string())?;

        writeln!(
            log_file,
            "Drop: launching {} with args {:?} in {}",
            command, args, install_dir
        )
        .map_err(|e| e.to_string())?;

        let launch_process = Command::new(command)
            .current_dir(install_dir)
            .stdout(log_file)
            .args(args)
            .spawn()
            .map_err(|v| v.to_string())?;

        self.processes.insert(game_id, launch_process);

        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub enum Platform {
    Windows,
    Linux,
}

pub type ProcessCompatabilityMatrix = HashMap<Platform, Vec<Platform>>;
pub static PROCESS_COMPATABILITY_MATRIX: LazyLock<ProcessCompatabilityMatrix> =
    LazyLock::new(|| {
        let mut matrix: ProcessCompatabilityMatrix = HashMap::new();

        matrix.insert(Platform::Windows, vec![Platform::Windows]);
        matrix.insert(Platform::Linux, vec![Platform::Linux]); // TODO: add Proton support

        return matrix;
    });
