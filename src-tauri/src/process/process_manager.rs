use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    sync::{Arc, Mutex},
    thread::spawn,
};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use shared_child::SharedChild;
use tauri::{AppHandle, Manager};
use umu_wrapper_lib::command_builder::UmuCommandBuilder;

use crate::{
    db::{ApplicationTransientStatus, GameDownloadStatus, DATA_ROOT_DIR},
    download_manager::downloadable_metadata::DownloadableMetadata,
    games::library::push_game_update,
    games::state::GameStatusManager,
    AppState, DB,
};

pub struct ProcessManager<'a> {
    current_platform: Platform,
    log_output_dir: PathBuf,
    processes: HashMap<DownloadableMetadata, Arc<SharedChild>>,
    app_handle: AppHandle,
    game_launchers: HashMap<(Platform, Platform), &'a (dyn ProcessHandler + Sync + Send + 'static)>,
}

impl ProcessManager<'_> {
    pub fn new(app_handle: AppHandle) -> Self {
        let root_dir_lock = DATA_ROOT_DIR.lock().unwrap();
        let log_output_dir = root_dir_lock.join("logs");
        drop(root_dir_lock);

        ProcessManager {
            current_platform: if cfg!(windows) {
                Platform::Windows
            } else {
                Platform::Linux
            },

            app_handle,
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
                (
                    (Platform::Linux, Platform::Windows),
                    &UMULauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
            ]),
        }
    }

    // There's no easy way to distinguish between an executable name with
    // spaces and it's arguments.
    // I think if we just join the install_dir to whatever the user provides us, we'll be alright
    // In future, we should have a separate field for executable name and it's arguments
    fn process_command(&self, install_dir: &String, raw_command: String) -> (PathBuf, Vec<String>) {
        // let command_components = raw_command.split(" ").collect::<Vec<&str>>();
        let root = raw_command;

        let install_dir = Path::new(install_dir);
        let absolute_exe = install_dir.join(root);

        /*
        let args = command_components[1..]
            .iter()
            .map(|v| v.to_string())
            .collect();
         */
        (absolute_exe, Vec::new())
    }
    pub fn kill_game(&mut self, meta: DownloadableMetadata) -> Result<(), io::Error> {
        return match self.processes.get(&meta) {
            Some(child) => {
                child.kill()?;
                child.wait()?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Game ID not running",
            )),
        };
    }

    fn on_process_finish(
        &mut self,
        meta: DownloadableMetadata,
        result: Result<ExitStatus, std::io::Error>,
    ) {
        if !self.processes.contains_key(&meta) {
            warn!("process on_finish was called, but game_id is no longer valid. finished with result: {:?}", result);
            return;
        }

        info!("process for {:?} exited with {:?}", meta, result);

        self.processes.remove(&meta);

        let mut db_handle = DB.borrow_data_mut().unwrap();
        db_handle.applications.transient_statuses.remove(&meta);

        let current_state = db_handle.applications.game_statuses.get(&meta.id).cloned();
        if let Some(saved_state) = current_state {
            if let GameDownloadStatus::SetupRequired {
                version_name,
                install_dir,
            } = saved_state
            {
                if let Ok(exit_code) = result {
                    if exit_code.success() {
                        db_handle.applications.game_statuses.insert(
                            meta.id.clone(),
                            GameDownloadStatus::Installed {
                                version_name: version_name.to_string(),
                                install_dir: install_dir.to_string(),
                            },
                        );
                    }
                }
            }
        }
        drop(db_handle);

        let status = GameStatusManager::fetch_state(&meta.id);

        push_game_update(&self.app_handle, &meta, status);

        // TODO better management
    }

    pub fn valid_platform(&self, platform: &Platform) -> Result<bool, String> {
        let current = &self.current_platform;
        Ok(self
            .game_launchers
            .contains_key(&(current.clone(), platform.clone())))
    }

    pub fn launch_process(&mut self, meta: DownloadableMetadata) -> Result<(), String> {
        if self.processes.contains_key(&meta) {
            return Err("Game or setup is already running.".to_owned());
        }

        let mut db_lock = DB.borrow_data_mut().unwrap();
        info!(
            "Launching process {:?} with games {:?}",
            meta, db_lock.applications.game_versions
        );

        let game_status = db_lock
            .applications
            .game_statuses
            .get(&meta.id)
            .ok_or("Game not installed")?;

        let status_metadata: Option<(&String, &String)> = match game_status {
            GameDownloadStatus::Installed {
                version_name,
                install_dir,
            } => Some((version_name, install_dir)),
            GameDownloadStatus::SetupRequired {
                version_name,
                install_dir,
            } => Some((version_name, install_dir)),
            _ => None,
        };

        if status_metadata.is_none() {
            return Err("Game has not been downloaded.".to_owned());
        }

        let (version_name, install_dir) = status_metadata.unwrap();

        let game_version = db_lock
            .applications
            .game_versions
            .get(&meta.id)
            .ok_or("Invalid game ID".to_owned())?
            .get(version_name)
            .ok_or("Invalid version name".to_owned())?;

        let raw_command: String = match game_status {
            GameDownloadStatus::Installed {
                version_name: _,
                install_dir: _,
            } => game_version.launch_command.clone(),
            GameDownloadStatus::SetupRequired {
                version_name: _,
                install_dir: _,
            } => game_version.setup_command.clone(),
            _ => panic!("unreachable code"),
        };

        let (command, args) = self.process_command(install_dir, raw_command);

        let target_current_dir = command.parent().unwrap().to_str().unwrap();

        info!(
            "launching process {} in {}",
            command.to_str().unwrap(),
            target_current_dir
        );

        let current_time = chrono::offset::Local::now();
        let log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(self.log_output_dir.join(format!(
                "{}-{}-{}.log",
                meta.id.clone(),
                meta.version.clone().unwrap_or_default(),
                current_time.timestamp()
            )))
            .map_err(|v| v.to_string())?;

        let error_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(self.log_output_dir.join(format!(
                "{}-{}-{}-error.log",
                meta.id.clone(),
                meta.version.clone().unwrap_or_default(),
                current_time.timestamp()
            )))
            .map_err(|v| v.to_string())?;

        let current_platform = self.current_platform.clone();
        let target_platform = game_version.platform.clone();

        let game_launcher = self
            .game_launchers
            .get(&(current_platform, target_platform))
            .ok_or("Invalid version for this platform.")
            .map_err(|e| e.to_string())?;

        let launch_process = game_launcher.launch_process(
            &meta,
            command.to_str().unwrap().to_owned(),
            args,
            &target_current_dir.to_string(),
            log_file,
            error_file,
        )?;

        let launch_process_handle =
            Arc::new(SharedChild::new(launch_process).map_err(|e| e.to_string())?);

        db_lock
            .applications
            .transient_statuses
            .insert(meta.clone(), ApplicationTransientStatus::Running {});

        push_game_update(
            &self.app_handle,
            &meta,
            (None, Some(ApplicationTransientStatus::Running {})),
        );

        let wait_thread_handle = launch_process_handle.clone();
        let wait_thread_apphandle = self.app_handle.clone();
        let wait_thread_game_id = meta.clone();

        spawn(move || {
            let result: Result<ExitStatus, std::io::Error> = launch_process_handle.wait();

            let app_state = wait_thread_apphandle.state::<Mutex<AppState>>();
            let app_state_handle = app_state.lock().unwrap();

            let mut process_manager_handle = app_state_handle.process_manager.lock().unwrap();
            process_manager_handle.on_process_finish(wait_thread_game_id, result);

            // As everything goes out of scope, they should get dropped
            // But just to explicit about it
            drop(process_manager_handle);
            drop(app_state_handle);
        });

        self.processes.insert(meta, wait_thread_handle);

        info!("finished spawning process");

        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Platform {
    Windows,
    Linux,
}

pub trait ProcessHandler: Send + 'static {
    fn launch_process(
        &self,
        meta: &DownloadableMetadata,
        command: String,
        args: Vec<String>,
        current_dir: &String,
        log_file: File,
        error_file: File,
    ) -> Result<Child, String>;
}

struct NativeGameLauncher;
impl ProcessHandler for NativeGameLauncher {
    fn launch_process(
        &self,
        meta: &DownloadableMetadata,
        command: String,
        args: Vec<String>,
        current_dir: &String,
        log_file: File,
        error_file: File,
    ) -> Result<Child, String> {
        Command::new(command)
            .current_dir(current_dir)
            .stdout(log_file)
            .stderr(error_file)
            .args(args)
            .spawn()
            .map_err(|v| v.to_string())
    }
}

const UMU_LAUNCHER_EXECUTABLE: &str = "umu-run";
struct UMULauncher;
impl ProcessHandler for UMULauncher {
    fn launch_process(
        &self,
        meta: &DownloadableMetadata,
        command: String,
        args: Vec<String>,
        current_dir: &String,
        log_file: File,
        error_file: File,
    ) -> Result<Child, String> {
        UmuCommandBuilder::new(UMU_LAUNCHER_EXECUTABLE, command)
            .game_id(String::from("0"))
            .launch_args(args)
            .build()
            .spawn()
            .map_err(|x| x.to_string())
    }
}
