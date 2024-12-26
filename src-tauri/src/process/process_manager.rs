use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::LazyLock,
};

use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    db::{GameStatus, DATA_ROOT_DIR},
    DB,
};

pub struct ProcessManager<'a> {
    current_platform: Platform,
    log_output_dir: PathBuf,
    processes: HashMap<String, Child>,
    game_launchers: HashMap<(Platform, Platform), &'a (dyn ProcessHandler + Sync + Send + 'static)>,
}

impl ProcessManager<'_> {
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
            game_launchers: HashMap::from([
                // Current platform to target platform
                (
                    (Platform::Windows, Platform::Windows),
                    &NativeGameLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                (
                    (Platform::Linux, Platform::Linux),
                    &NativeGameLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                /*
                (
                    (Platform::Linux, Platform::Windows),
                    &UMULauncher {} as &(dyn ProcessHandler + Sync + Send + 'static)
                )
                 */
            ]),
        }
    }

    fn process_command(&self, install_dir: &String, raw_command: String) -> (String, Vec<String>) {
        let command_components = raw_command.split(" ").collect::<Vec<&str>>();
        let root = command_components[0].to_string();

        let install_dir = Path::new(install_dir);
        let absolute_exe = install_dir.join(root);

        let args = command_components[1..]
            .into_iter()
            .map(|v| v.to_string())
            .collect();
        (absolute_exe.to_str().unwrap().to_owned(), args)
    }

    pub fn valid_platform(&self, platform: &Platform) -> Result<bool, String> {
        let current = &self.current_platform;
        Ok(self
            .game_launchers
            .contains_key(&(current.clone(), platform.clone())))
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

        let (command, args) =
            self.process_command(install_dir, game_version.launch_command.clone());

        info!("launching process {} in {}", command, install_dir);

        let current_time = chrono::offset::Local::now();
        let mut log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(
                self.log_output_dir
                    .join(format!("{}-{}.log", game_id, current_time.timestamp())),
            )
            .map_err(|v| v.to_string())?;

        let mut error_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(self.log_output_dir.join(format!(
                "{}-{}-error.log",
                game_id,
                current_time.timestamp()
            )))
            .map_err(|v| v.to_string())?;

        info!("opened log file for {}", command);

        let current_platform = self.current_platform.clone();
        let target_platform = game_version.platform.clone();

        let game_launcher = self
            .game_launchers
            .get(&(current_platform, target_platform))
            .ok_or("Invalid version for this platform.")
            .map_err(|e| e.to_string())?;

        let launch_process = game_launcher.launch_game(
            &game_id,
            version_name,
            command,
            args,
            install_dir,
            log_file,
            error_file,
        )?;

        self.processes.insert(game_id, launch_process);

        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Platform {
    Windows,
    Linux,
}

pub trait ProcessHandler: Send + 'static {
    fn launch_game(
        &self,
        game_id: &String,
        version_name: &String,
        command: String,
        args: Vec<String>,
        install_dir: &String,
        log_file: File,
        error_file: File,
    ) -> Result<Child, String>;
}

struct NativeGameLauncher;
impl ProcessHandler for NativeGameLauncher {
    fn launch_game(
        &self,
        game_id: &String,
        version_name: &String,
        command: String,
        args: Vec<String>,
        install_dir: &String,
        log_file: File,
        error_file: File,
    ) -> Result<Child, String> {
        Command::new(command)
            .current_dir(install_dir)
            .stdout(log_file)
            .stderr(error_file)
            .args(args)
            .spawn()
            .map_err(|v| v.to_string())
    }
}

struct UMULauncher;
impl ProcessHandler for UMULauncher {
    fn launch_game(
        &self,
        game_id: &String,
        version_name: &String,
        command: String,
        args: Vec<String>,
        install_dir: &String,
        log_file: File,
        error_file: File,
    ) -> Result<Child, String> {
        todo!()
    }
}
