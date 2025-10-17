use std::{
    collections::HashMap,
    fs::{OpenOptions, create_dir_all},
    io,
    path::PathBuf,
    process::{Command, ExitStatus},
    str::FromStr,
    sync::Arc,
    thread::spawn,
    time::{Duration, SystemTime},
};

use database::{
    ApplicationTransientStatus, Database, DownloadType, DownloadableMetadata, GameDownloadStatus,
    GameVersion, borrow_db_checked, borrow_db_mut_checked, db::DATA_ROOT_DIR, platform::Platform,
};
use dynfmt::Format;
use dynfmt::SimpleCurlyFormat;
use games::{library::push_game_update, state::GameStatusManager};
use log::{debug, info, warn};
use shared_child::SharedChild;
use tauri::AppHandle;

use crate::{
    PROCESS_MANAGER,
    error::ProcessError,
    format::DropFormatArgs,
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
                    &NativeGameLauncher {} as &(dyn ProcessHandler + Sync + Send + 'static),
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
                process.handle.wait()?;
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
            return Err(ProcessError::FailedLaunch(game_id));
            // let _ = self.app_handle.emit("launch_external_error", &game_id);
        }

        let version_data = match db_handle.applications.game_versions.get(&game_id) {
            // This unwrap here should be resolved by just making the hashmap accept an option rather than just a String
            Some(res) => res.get(&meta.version.unwrap()).expect("Failed to get game version from installed game versions. Is the database corrupted?"),
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

    /// Must be called through spawn as it is currently blocking
    pub fn launch_process(&mut self, game_id: String) -> Result<(), ProcessError> {
        if self.processes.contains_key(&game_id) {
            return Err(ProcessError::AlreadyRunning);
        }

        let version = match borrow_db_checked()
            .applications
            .game_statuses
            .get(&game_id)
            .cloned()
        {
            Some(GameDownloadStatus::Installed { version_name, .. }) => version_name,
            Some(GameDownloadStatus::SetupRequired { version_name, .. }) => version_name,
            _ => return Err(ProcessError::NotInstalled),
        };
        let meta = DownloadableMetadata {
            id: game_id.clone(),
            version: Some(version.clone()),
            download_type: DownloadType::Game,
        };

        let mut db_lock = borrow_db_mut_checked();

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
            .get(&game_id)
            .ok_or(ProcessError::InvalidID)?
            .get(version_name)
            .ok_or(ProcessError::InvalidVersion)?;

        // TODO: refactor this path with open_process_logs
        let game_log_folder = &self.get_log_dir(game_id);
        create_dir_all(game_log_folder).map_err(ProcessError::IOError)?;

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

        let target_platform = game_version.platform;

        let process_handler = self.fetch_process_handler(&db_lock, &target_platform)?;

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

        #[allow(clippy::unwrap_used)]
        let launch = PathBuf::from_str(install_dir).unwrap().join(launch);
        let launch = launch.display().to_string();

        let launch_string = process_handler.create_launch_process(
            &meta,
            launch.to_string(),
            args.clone(),
            game_version,
            install_dir,
        )?;

        let format_args = DropFormatArgs::new(
            launch_string,
            install_dir,
            &game_version.launch_command,
            launch.to_string(),
        );

        let launch_string = SimpleCurlyFormat
            .format(&game_version.launch_command_template, format_args)
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

        debug!("final launch string:\n\n{launch_string}\n");

        command
            .stderr(error_file)
            .stdout(log_file)
            .env_remove("RUST_LOG")
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
        args: Vec<String>,
        game_version: &GameVersion,
        current_dir: &str,
    ) -> Result<String, ProcessError>;

    fn valid_for_platform(&self, db: &Database, target: &Platform) -> bool;
}
