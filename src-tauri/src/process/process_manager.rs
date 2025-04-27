use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, Error},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::spawn,
};

use dynfmt::Format;
use dynfmt::SimpleCurlyFormat;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use shared_child::SharedChild;
use tauri::{AppHandle, Manager};
use umu_wrapper_lib::command_builder::UmuCommandBuilder;

use crate::{
    database::db::{
        borrow_db_mut_checked, ApplicationTransientStatus, GameDownloadStatus, GameVersion,
        DATA_ROOT_DIR,
    },
    download_manager::downloadable_metadata::{DownloadType, DownloadableMetadata},
    error::process_error::ProcessError,
    games::{library::push_game_update, state::GameStatusManager},
    AppState, DB,
};

pub struct ProcessManager<'a> {
    current_platform: Platform,
    log_output_dir: PathBuf,
    processes: HashMap<String, Arc<SharedChild>>,
    app_handle: AppHandle,
    game_launchers: HashMap<(Platform, Platform), &'a (dyn ProcessHandler + Sync + Send + 'static)>,
}

impl ProcessManager<'_> {
    pub fn new(app_handle: AppHandle) -> Self {
        let root_dir_lock = DATA_ROOT_DIR.lock().unwrap();
        let log_output_dir = root_dir_lock.join("logs");
        drop(root_dir_lock);

        ProcessManager {
            #[cfg(target_os = "windows")]
            current_platform: Platform::Windows,

            #[cfg(target_os = "macos")]
            current_platform: Platform::macOS,

            #[cfg(target_os = "linux")]
            current_platform: Platform::Linux,

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
                    (Platform::macOS, Platform::macOS),
                    &NativeGameLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                (
                    (Platform::Linux, Platform::Windows),
                    &UMULauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
            ]),
        }
    }

    pub fn kill_game(&mut self, game_id: String) -> Result<(), io::Error> {
        match self.processes.get(&game_id) {
            Some(child) => {
                child.kill()?;
                child.wait()?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Game ID not running",
            )),
        }
    }

    fn on_process_finish(&mut self, game_id: String, result: Result<ExitStatus, std::io::Error>) {
        if !self.processes.contains_key(&game_id) {
            warn!("process on_finish was called, but game_id is no longer valid. finished with result: {:?}", result);
            return;
        }

        debug!("process for {:?} exited with {:?}", &game_id, result);

        self.processes.remove(&game_id);

        let mut db_handle = borrow_db_mut_checked();
        let meta = db_handle
            .applications
            .installed_game_version
            .get(&game_id)
            .cloned()
            .unwrap();
        db_handle.applications.transient_statuses.remove(&meta);

        let current_state = db_handle.applications.game_statuses.get(&game_id).cloned();
        if let Some(saved_state) = current_state {
            if let GameDownloadStatus::SetupRequired {
                version_name,
                install_dir,
            } = saved_state
            {
                if let Ok(exit_code) = result {
                    if exit_code.success() {
                        db_handle.applications.game_statuses.insert(
                            game_id.clone(),
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

        let status = GameStatusManager::fetch_state(&game_id);

        push_game_update(&self.app_handle, &game_id, status);

        // TODO better management
    }

    pub fn valid_platform(&self, platform: &Platform) -> Result<bool, String> {
        let current = &self.current_platform;
        Ok(self
            .game_launchers
            .contains_key(&(current.clone(), platform.clone())))
    }

    pub fn launch_process(&mut self, game_id: String) -> Result<(), ProcessError> {
        if self.processes.contains_key(&game_id) {
            return Err(ProcessError::AlreadyRunning);
        }

        let version = match DB
            .borrow_data()
            .unwrap()
            .applications
            .game_statuses
            .get(&game_id)
            .cloned()
        {
            Some(GameDownloadStatus::Installed { version_name, .. }) => version_name,
            Some(GameDownloadStatus::SetupRequired { .. }) => {
                return Err(ProcessError::SetupRequired)
            }
            _ => return Err(ProcessError::NotInstalled),
        };
        let meta = DownloadableMetadata {
            id: game_id.clone(),
            version: Some(version.clone()),
            download_type: DownloadType::Game,
        };

        let mut db_lock = borrow_db_mut_checked();
        debug!(
            "Launching process {:?} with games {:?}",
            &game_id, db_lock.applications.game_versions
        );

        let game_status = db_lock
            .applications
            .game_statuses
            .get(&game_id)
            .ok_or(ProcessError::NotInstalled)?;

        let (version_name, install_dir) = match game_status {
            GameDownloadStatus::Installed {
                version_name,
                install_dir,
            } => (version_name, install_dir),
            GameDownloadStatus::SetupRequired {
                version_name,
                install_dir,
            } => (version_name, install_dir),
            _ => return Err(ProcessError::NotDownloaded),
        };

        let game_version = db_lock
            .applications
            .game_versions
            .get(&game_id)
            .ok_or(ProcessError::InvalidID)?
            .get(version_name)
            .ok_or(ProcessError::InvalidVersion)?;

        let current_time = chrono::offset::Local::now();
        let log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(self.log_output_dir.join(format!(
                "{}-{}-{}.log",
                &game_id,
                &version,
                current_time.timestamp()
            )))
            .map_err(ProcessError::IOError)?;

        let error_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(self.log_output_dir.join(format!(
                "{}-{}-{}-error.log",
                &game_id,
                &version,
                current_time.timestamp()
            )))
            .map_err(ProcessError::IOError)?;

        let current_platform = self.current_platform.clone();
        let target_platform = game_version.platform.clone();

        let game_launcher = self
            .game_launchers
            .get(&(current_platform, target_platform))
            .ok_or(ProcessError::InvalidPlatform)?;

        let (launch, args) = match game_status {
            GameDownloadStatus::Installed {
                version_name: _,
                install_dir: _,
            } => (&game_version.launch_command, &game_version.launch_args),
            GameDownloadStatus::SetupRequired {
                version_name: _,
                install_dir: _,
            } => (&game_version.setup_command, &game_version.setup_args),
            GameDownloadStatus::Remote {} => unreachable!("nuh uh"),
        };

        let launch = PathBuf::from_str(&install_dir).unwrap().join(launch);
        let launch = launch.to_str().unwrap();

        let launch_string = game_launcher.create_launch_process(
            &meta,
            launch.to_string(),
            args.to_vec(),
            game_version,
            install_dir,
        );

        let launch_string = SimpleCurlyFormat
            .format(&game_version.launch_command_template, &[launch_string])
            .map_err(|e| ProcessError::FormatError(e.to_string()))?
            .to_string();

        #[cfg(target_os = "windows")]
        let mut command = Command::new("cmd");
        #[cfg(target_os = "windows")]
        command.args(["/C", &launch_string]);

        info!("launching (in {}): {}", install_dir, launch_string,);

        #[cfg(unix)]
        let mut command: Command = Command::new("sh");
        #[cfg(unix)]
        command.arg("-c").arg(launch_string);

        command
            .stderr(error_file)
            .stdout(log_file)
            .current_dir(install_dir);

        let child = command.spawn().map_err(ProcessError::IOError)?;

        let launch_process_handle =
            Arc::new(SharedChild::new(child).map_err(ProcessError::IOError)?);

        db_lock
            .applications
            .transient_statuses
            .insert(meta.clone(), ApplicationTransientStatus::Running {});

        push_game_update(
            &self.app_handle,
            &meta.id,
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
            process_manager_handle.on_process_finish(wait_thread_game_id.id, result);

            // As everything goes out of scope, they should get dropped
            // But just to explicit about it
            drop(process_manager_handle);
            drop(app_state_handle);
        });

        self.processes.insert(meta.id, wait_thread_handle);
        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Platform {
    Windows,
    Linux,
    macOS,
}

pub trait ProcessHandler: Send + 'static {
    fn create_launch_process(
        &self,
        meta: &DownloadableMetadata,
        launch_command: String,
        args: Vec<String>,
        game_version: &GameVersion,
        current_dir: &str,
    ) -> String;
}

struct NativeGameLauncher;
impl ProcessHandler for NativeGameLauncher {
    fn create_launch_process(
        &self,
        _meta: &DownloadableMetadata,
        launch_command: String,
        args: Vec<String>,
        game_version: &GameVersion,
        current_dir: &str,
    ) -> String {
        format!("\"{}\" {}", launch_command, args.join(" "))
    }
}

const UMU_LAUNCHER_EXECUTABLE: &str = "umu-run";
struct UMULauncher;
impl ProcessHandler for UMULauncher {
    fn create_launch_process(
        &self,
        _meta: &DownloadableMetadata,
        launch_command: String,
        args: Vec<String>,
        game_version: &GameVersion,
        _current_dir: &str,
    ) -> String {
        debug!("Game override: \"{:?}\"", &game_version.umu_id_override);
        let game_id = match &game_version.umu_id_override {
            Some(game_override) => game_override
                .is_empty()
                .then_some(game_version.game_id.clone())
                .unwrap_or(game_override.clone()),
            None => game_version.game_id.clone(),
        };
        format!(
            "GAMEID={game_id} {umu} \"{launch}\" {args}",
            umu = UMU_LAUNCHER_EXECUTABLE,
            launch = launch_command,
            args = args.join(" ")
        )
    }
}
