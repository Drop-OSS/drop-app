use std::{
    collections::HashMap,
    fs::{OpenOptions, create_dir_all},
    io,
    path::PathBuf,
    process::{Command, ExitStatus},
    sync::Arc,
    thread::spawn,
    time::{Duration, SystemTime},
};

use database::{
    ApplicationTransientStatus, Database, DownloadableMetadata, GameDownloadStatus, GameVersion,
    borrow_db_checked, borrow_db_mut_checked, db::DATA_ROOT_DIR, platform::Platform,
};
use dynfmt::Format;
use dynfmt::SimpleCurlyFormat;
use games::{library::push_game_update, state::GameStatusManager};
use log::{debug, info, warn};
use serde::Serialize;
use shared_child::SharedChild;
use tauri::{AppHandle, Emitter as _};

use crate::{
    PROCESS_MANAGER,
    error::ProcessError,
    format::DropFormatArgs,
    parser::{LaunchParameters, ParsedCommand},
    process_handlers::{AsahiMuvmLauncher, NativeGameLauncher, UMULauncher},
};

pub struct RunningProcess {
    handle: Arc<SharedChild>,
    start: SystemTime,
    manually_killed: bool,
}

pub struct ProcessManager<'a> {
    current_platform: Platform,
    log_output_dir: PathBuf,
    processes: HashMap<String, RunningProcess>,
    game_launchers: Vec<(
        (Platform, Platform),
        &'a (dyn ProcessHandler + Sync + Send + 'static),
    )>,
    app_handle: AppHandle,
}

#[derive(Serialize)]
pub struct LaunchOption {
    name: String,
}

impl ProcessManager<'_> {
    pub fn new(app_handle: AppHandle) -> Self {
        let log_output_dir = DATA_ROOT_DIR.join("logs");

        ProcessManager {
            #[cfg(target_os = "windows")]
            current_platform: Platform::Windows,

            #[cfg(target_os = "macos")]
            current_platform: Platform::macOS,

            #[cfg(target_os = "linux")]
            current_platform: Platform::Linux,

            processes: HashMap::new(),
            log_output_dir,
            game_launchers: vec![
                // Current platform to target platform
                (
                    (Platform::Windows, Platform::Windows),
                    &NativeGameLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                (
                    (Platform::Linux, Platform::Linux),
                    &UMULauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                (
                    (Platform::macOS, Platform::macOS),
                    &NativeGameLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                (
                    (Platform::Linux, Platform::Windows),
                    &AsahiMuvmLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
                (
                    (Platform::Linux, Platform::Windows),
                    &UMULauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
                ),
            ],
            app_handle,
        }
    }

    pub fn kill_game(&mut self, game_id: String) -> Result<(), io::Error> {
        match self.processes.get_mut(&game_id) {
            Some(process) => {
                process.manually_killed = true;
                process.handle.kill()?;
                let exit_status = process.handle.wait()?;
                info!("exit status: {:?}", exit_status);
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Game ID not running",
            )),
        }
    }

    pub fn get_log_dir(&self, game_id: String) -> PathBuf {
        self.log_output_dir.join(game_id)
    }

    fn on_process_finish(
        &mut self,
        game_id: String,
        result: Result<ExitStatus, std::io::Error>,
    ) -> Result<(), ProcessError> {
        if !self.processes.contains_key(&game_id) {
            warn!(
                "process on_finish was called, but game_id is no longer valid. finished with result: {result:?}"
            );
            return Ok(());
        }

        debug!("process for {:?} exited with {:?}", &game_id, result);

        let process = match self.processes.remove(&game_id) {
            Some(process) => process,
            None => {
                info!("Attempted to stop process {game_id} which didn't exist");
                return Ok(());
            }
        };

        let mut db_handle = borrow_db_mut_checked();
        let meta = db_handle
            .applications
            .installed_game_version
            .get(&game_id)
            .cloned()
            .unwrap_or_else(|| panic!("Could not get installed version of {}", &game_id));
        db_handle.applications.transient_statuses.remove(&meta);

        let current_state = db_handle.applications.game_statuses.get(&game_id).cloned();
        if let Some(GameDownloadStatus::SetupRequired {
            version_name,
            install_dir,
        }) = current_state
            && let Ok(exit_code) = result
            && exit_code.success()
        {
            db_handle.applications.game_statuses.insert(
                game_id.clone(),
                GameDownloadStatus::Installed {
                    version_name: version_name.to_string(),
                    install_dir: install_dir.to_string(),
                },
            );
        }

        let elapsed = process.start.elapsed().unwrap_or(Duration::ZERO);
        // If we started and ended really quickly, something might've gone wrong
        // Or if the status isn't 0
        // Or if it's an error
        if !process.manually_killed
            && (elapsed.as_secs() <= 2 || result.map_or(true, |r| !r.success()))
        {
            warn!("drop detected that the game {game_id} may have failed to launch properly");
            let _ = self.app_handle.emit("launch_external_error", &game_id);
        }

        let version_data = match db_handle.applications.game_versions.get(&meta.version) {
            // This unwrap here should be resolved by just making the hashmap accept an option rather than just a String
            Some(res) => res,
            None => todo!(),
        };

        let status = GameStatusManager::fetch_state(&game_id, &db_handle);

        push_game_update(
            &self.app_handle,
            &game_id,
            Some(version_data.clone()),
            status,
        );
        Ok(())
    }

    fn fetch_process_handler(
        &self,
        db_lock: &Database,
        target_platform: &Platform,
    ) -> Result<&(dyn ProcessHandler + Send + Sync), ProcessError> {
        Ok(self
            .game_launchers
            .iter()
            .find(|e| {
                let (e_current, e_target) = e.0;
                e_current == self.current_platform
                    && e_target == *target_platform
                    && e.1.valid_for_platform(db_lock, target_platform)
            })
            .ok_or(ProcessError::InvalidPlatform)?
            .1)
    }

    pub fn valid_platform(&self, platform: &Platform) -> bool {
        let db_lock = borrow_db_checked();
        let process_handler = self.fetch_process_handler(&db_lock, platform);
        process_handler.is_ok()
    }

    pub fn get_launch_options(game_id: String) -> Result<Vec<LaunchOption>, ProcessError> {
        let db_lock = borrow_db_checked();

        let meta = db_lock
            .applications
            .installed_game_version
            .get(&game_id)
            .cloned()
            .ok_or(ProcessError::NotInstalled)?;

        let game_version = db_lock
            .applications
            .game_versions
            .get(&meta.version)
            .ok_or(ProcessError::InvalidVersion)?;

        let launch_options = game_version
            .launches
            .iter()
            .filter(|v| v.platform == meta.target_platform)
            .map(|v| LaunchOption {
                name: v.name.clone(),
            })
            .collect::<Vec<LaunchOption>>();

        Ok(launch_options)
    }

    pub fn launch_process(
        &mut self,
        game_id: String,
        launch_process_index: usize,
    ) -> Result<(), ProcessError> {
        if self.processes.contains_key(&game_id) {
            return Err(ProcessError::AlreadyRunning);
        }

        let mut db_lock = borrow_db_mut_checked();

        let meta = db_lock
            .applications
            .installed_game_version
            .get(&game_id)
            .cloned()
            .ok_or(ProcessError::NotInstalled)?;

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
            _ => return Err(ProcessError::NotInstalled),
        };

        debug!(
            "Launching process {:?} with version {:?}",
            &game_id,
            db_lock.applications.game_versions.get(&game_id)
        );

        let game_version = db_lock
            .applications
            .game_versions
            .get(version_name)
            .ok_or(ProcessError::InvalidVersion)?;

        // TODO: refactor this path with open_process_logs
        let game_log_folder = &self.get_log_dir(game_id);
        create_dir_all(game_log_folder)?;

        let current_time = chrono::offset::Local::now();
        let log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(game_log_folder.join(format!(
                "{}-{}.log",
                &meta.version,
                current_time.timestamp()
            )))?;

        let error_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true)
            .open(game_log_folder.join(format!(
                "{}-{}-error.log",
                &meta.version,
                current_time.timestamp()
            )))?;

        let target_platform = meta.target_platform;

        let process_handler = self.fetch_process_handler(&db_lock, &target_platform)?;

        let (target_command, executor) = match game_status {
            GameDownloadStatus::Installed {
                version_name: _,
                install_dir: _,
            } => {
                let (_, launch_config) = game_version
                    .launches
                    .iter()
                    .filter(|v| v.platform == target_platform)
                    .enumerate()
                    .find(|(i, _)| *i == launch_process_index)
                    .ok_or(ProcessError::NotInstalled)?;
                (
                    launch_config.command.clone(),
                    launch_config.executor.as_ref(),
                )
            }
            GameDownloadStatus::SetupRequired {
                version_name: _,
                install_dir: _,
            } => {
                let setup_config = game_version
                    .setups
                    .iter()
                    .find(|v| v.platform == target_platform)
                    .ok_or(ProcessError::NotInstalled)?;

                (setup_config.command.clone(), None)
            }
            _ => unreachable!("Game registered as 'Partially Installed'"),
        };

        let target_command = ParsedCommand::parse(target_command)?;

        let launch_parameters = if let Some(executor) = executor {
            let err = ProcessError::RequiredDependency(
                executor.game_id.clone(),
                executor.version_id.clone(),
            );

            let executor_metadata = db_lock
                .applications
                .installed_game_version
                .get(&executor.game_id)
                .ok_or(err.clone())?;

            let executor_game_status = db_lock
                .applications
                .game_statuses
                .get(&executor.game_id)
                .ok_or(err.clone())?;

            let executor_install_dir = match executor_game_status {
                GameDownloadStatus::Installed {
                    version_name: _,
                    install_dir,
                } => Ok(install_dir),
                GameDownloadStatus::SetupRequired {
                    version_name: _,
                    install_dir: _,
                } => todo!(),
                _ => Err(err.clone()),
            }?;

            let executor_game_version = db_lock
                .applications
                .game_versions
                .get(&executor.version_id)
                .ok_or(err.clone())?;

            let executor_launch_config = executor_game_version
                .launches
                .iter()
                .find(|v| v.launch_id == executor.launch_id)
                .ok_or(err)?;

            println!("{}", executor_launch_config.command);
            let mut exe_command = ParsedCommand::parse(executor_launch_config.command.clone())?;
            println!("{:?}", exe_command);
            exe_command.env.extend(target_command.env);
            exe_command.make_absolute(executor_install_dir.into());

            exe_command.args.iter_mut().for_each(|v| {
                *v = v.replace("{executor}", &target_command.command);
            });

            let executor_launch_string = process_handler.create_launch_process(
                executor_metadata,
                exe_command.reconstruct(),
                executor_game_version,
                install_dir,
            )?;

            LaunchParameters(executor_launch_string, install_dir.into())
        } else {
            let target_launch_string = process_handler.create_launch_process(
                &meta,
                target_command.reconstruct(),
                game_version,
                install_dir,
            )?;

            let mut parsed_launch = ParsedCommand::parse(target_launch_string.clone())?;
            let executable_name = parsed_launch.command.clone();
            parsed_launch.make_absolute(install_dir.into());

            let format_args = DropFormatArgs::new(
                target_launch_string,
                install_dir,
                &executable_name,
                parsed_launch.command,
                None,
            );

            let target_launch_string = SimpleCurlyFormat
                .format(&game_version.launch_template, &format_args)
                .map_err(|e| ProcessError::FormatError(e.to_string()))?
                .to_string();

            let target_launch_string = SimpleCurlyFormat
                .format(&target_launch_string, format_args)
                .map_err(|e| ProcessError::FormatError(e.to_string()))?
                .to_string();

            LaunchParameters(target_launch_string, install_dir.into())
        };

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;
        #[cfg(target_os = "windows")]
        let mut command = Command::new("cmd");
        #[cfg(target_os = "windows")]
        command.raw_arg(format!("/C \"{}\"", &launch_parameters.0));

        info!(
            "launching (in {}): {}",
            launch_parameters.1.to_string_lossy(),
            launch_parameters.0
        );

        #[cfg(unix)]
        let mut command: Command = Command::new("sh");
        #[cfg(unix)]
        command.args(vec!["-c", &launch_parameters.0]);

        command
            .stderr(error_file)
            .stdout(log_file)
            .env_remove("RUST_LOG")
            .current_dir(launch_parameters.1);

        let child = command.spawn()?;

        let launch_process_handle =
            Arc::new(SharedChild::new(child)?);

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
        let wait_thread_game_id = meta.clone();

        self.processes.insert(
            meta.id,
            RunningProcess {
                handle: wait_thread_handle,
                start: SystemTime::now(),
                manually_killed: false,
            },
        );
        spawn(move || {
            let result: Result<ExitStatus, std::io::Error> = launch_process_handle.wait();

            PROCESS_MANAGER
                .lock()
                .on_process_finish(wait_thread_game_id.id, result)
        });
        Ok(())
    }
}

pub trait ProcessHandler: Send + 'static {
    fn create_launch_process(
        &self,
        meta: &DownloadableMetadata,
        launch_command: String,
        game_version: &GameVersion,
        current_dir: &str,
    ) -> Result<String, ProcessError>;

    fn valid_for_platform(&self, db: &Database, target: &Platform) -> bool;
}
