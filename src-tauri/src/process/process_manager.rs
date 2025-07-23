use std::{
    collections::HashMap,
    fs::{create_dir_all, OpenOptions},
    io::{self},
    path::PathBuf,
    process::{Command, ExitStatus},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::spawn,
    time::{Duration, SystemTime},
};

use dynfmt::Format;
use dynfmt::SimpleCurlyFormat;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use shared_child::SharedChild;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{
    database::{
        db::{borrow_db_mut_checked, DATA_ROOT_DIR},
        models::data::{
            ApplicationTransientStatus, DownloadType, DownloadableMetadata, GameDownloadStatus,
            GameVersion,
        },
    },
    error::process_error::ProcessError,
    games::{library::push_game_update, state::GameStatusManager},
    AppState, DB,
};

pub struct RunningProcess {
    handle: Arc<SharedChild>,
    start: SystemTime,
    manually_killed: bool
}

pub struct ProcessManager<'a> {
    current_platform: Platform,
    log_output_dir: PathBuf,
    processes: HashMap<String, RunningProcess>,
    app_handle: AppHandle,
    game_launchers: HashMap<(Platform, Platform), &'a (dyn ProcessHandler + Sync + Send + 'static)>,
}

impl ProcessManager<'_> {
    pub fn new(app_handle: AppHandle) -> Self {
        let log_output_dir = DATA_ROOT_DIR.join("logs");

        ProcessManager {
            #[cfg(target_os = "windows")]
            current_platform: Platform::Windows,

            #[cfg(target_os = "macos")]
            current_platform: Platform::MacOs,

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
                    (Platform::MacOs, Platform::MacOs),
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
        match self.processes.get_mut(&game_id) {
            Some(process) => {
                process.manually_killed = true;
                process.handle.kill()?;
                process.handle.wait()?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Game ID not running",
            )),
        }
    }

    pub fn open_process_logs(&mut self, game_id: String) -> Result<(), ProcessError> {
        let dir = self.log_output_dir.join(game_id);
        self.app_handle
            .opener()
            .open_path(dir.to_str().unwrap(), None::<&str>)
            .map_err(|e| ProcessError::OpenerError(e))?;
        Ok(())
    }

    fn on_process_finish(&mut self, game_id: String, result: Result<ExitStatus, std::io::Error>) {
        if !self.processes.contains_key(&game_id) {
            warn!("process on_finish was called, but game_id is no longer valid. finished with result: {result:?}");
            return;
        }

        debug!("process for {:?} exited with {:?}", &game_id, result);

        let process = self.processes.remove(&game_id).unwrap();

        let mut db_handle = borrow_db_mut_checked();
        let meta = db_handle
            .applications
            .installed_game_version
            .get(&game_id)
            .cloned()
            .unwrap();
        db_handle.applications.transient_statuses.remove(&meta);

        let current_state = db_handle.applications.game_statuses.get(&game_id).cloned();
        if let Some(GameDownloadStatus::SetupRequired {
            version_name,
            install_dir,
        }) = current_state
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
        drop(db_handle);

        let elapsed = process.start.elapsed().unwrap_or(Duration::ZERO);
        // If we started and ended really quickly, something might've gone wrong
        // Or if the status isn't 0
        // Or if it's an error
        if !process.manually_killed && (elapsed.as_secs() <= 2 || result.is_err() || !result.unwrap().success()) {
            warn!("drop detected that the game {game_id} may have failed to launch properly");
            let _ = self.app_handle.emit("launch_external_error", &game_id);
        }

        let status = GameStatusManager::fetch_state(&game_id);
        push_game_update(&self.app_handle, &game_id, None, status);
    }

    pub fn valid_platform(&self, platform: &Platform) -> Result<bool, String> {
        let current = &self.current_platform;
        Ok(self.game_launchers.contains_key(&(*current, *platform)))
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
            GameDownloadStatus::PartiallyInstalled {
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

        // TODO: refactor this path with open_process_logs
        let game_log_folder = &self.log_output_dir.join(game_id);
        create_dir_all(game_log_folder).map_err(|e| ProcessError::IOError(e))?;

        let current_time = chrono::offset::Local::now();
        let log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(game_log_folder.join(format!("{}-{}.log", &version, current_time.timestamp())))
            .map_err(ProcessError::IOError)?;

        let error_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(game_log_folder.join(format!(
                "{}-{}-error.log",
                &version,
                current_time.timestamp()
            )))
            .map_err(ProcessError::IOError)?;

        let current_platform = self.current_platform;
        let target_platform = game_version.platform;

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
            GameDownloadStatus::PartiallyInstalled {
                version_name: _,
                install_dir: _,
            } => unreachable!("Game registered as 'Partially Installed'"),
            GameDownloadStatus::Remote {} => unreachable!("Game registered as 'Remote'"),
        };

        let launch = PathBuf::from_str(install_dir).unwrap().join(launch);
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
        use std::os::windows::process::CommandExt;

        #[cfg(target_os = "windows")]
        let mut command = Command::new("cmd");
        #[cfg(target_os = "windows")]
        command.raw_arg(format!("/C \"{}\"", &launch_string));

        info!("launching (in {install_dir}): {launch_string}",);

        #[cfg(unix)]
        let mut command: Command = Command::new("sh");
        #[cfg(unix)]
        command.args(vec!["-c", &launch_string]);

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
            None,
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

        self.processes.insert(
            meta.id,
            RunningProcess {
                handle: wait_thread_handle,
                start: SystemTime::now(),
                manually_killed: false
            },
        );
        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Platform {
    Windows,
    Linux,
    MacOs,
}

impl Platform {
    #[cfg(target_os = "windows")]
    pub const HOST: Platform = Self::Windows;
    #[cfg(target_os = "macos")]
    pub const HOST: Platform = Self::MacOs;
    #[cfg(target_os = "linux")]
    pub const HOST: Platform = Self::Linux;

    pub fn is_case_sensitive(&self) -> bool {
        match self {
            Self::Windows | Self::MacOs => false,
            Self::Linux => true,
        }
    }
}

impl From<&str> for Platform {
    fn from(value: &str) -> Self {
        match value.to_lowercase().trim() {
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "mac" | "macos" => Self::MacOs,
            _ => unimplemented!(),
        }
    }
}

impl From<whoami::Platform> for Platform {
    fn from(value: whoami::Platform) -> Self {
        match value {
            whoami::Platform::Windows => Platform::Windows,
            whoami::Platform::Linux => Platform::Linux,
            whoami::Platform::MacOS => Platform::MacOs,
            _ => unimplemented!(),
        }
    }
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
        _game_version: &GameVersion,
        _current_dir: &str,
    ) -> String {
        format!("\"{}\" {}", launch_command, args.join(" "))
    }
}

pub const UMU_LAUNCHER_EXECUTABLE: &str = "umu-run";
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
            Some(game_override) => {
                if game_override.is_empty() {
                    game_version.game_id.clone()
                } else {
                    game_override.clone()
                }
            }
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
