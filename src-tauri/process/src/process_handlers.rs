use std::fs::create_dir_all;

use client::compat::{COMPAT_INFO, UMU_LAUNCHER_EXECUTABLE};
use database::{
    Database, DownloadableMetadata, GameVersion, db::DATA_ROOT_DIR, platform::Platform,
};

use crate::{error::ProcessError, process_manager::ProcessHandler};

pub struct NativeGameLauncher;
impl ProcessHandler for NativeGameLauncher {
    fn create_launch_process(
        &self,
        _meta: &DownloadableMetadata,
        launch_command: String,
        _game_version: &GameVersion,
        _current_dir: &str,
    ) -> Result<String, ProcessError> {
        Ok(format!("\"{}\"", launch_command))
    }

    fn valid_for_platform(&self, _db: &Database, _target: &Platform) -> bool {
        true
    }
}

pub struct UMULauncher;
impl ProcessHandler for UMULauncher {
    fn create_launch_process(
        &self,
        meta: &DownloadableMetadata,
        launch_command: String,
        game_version: &GameVersion,
        _current_dir: &str,
    ) -> Result<String, ProcessError> {
        let launch_config = game_version
            .launches
            .iter()
            .find(|v| v.platform == meta.target_platform)
            .ok_or(ProcessError::NotInstalled)?;

        let game_id = match &launch_config.umu_id_override {
            Some(game_override) => {
                if game_override.is_empty() {
                    game_version.version_id.clone()
                } else {
                    game_override.clone()
                }
            }
            None => game_version.version_id.clone(),
        };
        let pfx_dir = DATA_ROOT_DIR.join("pfx");
        let pfx_dir = pfx_dir.join(meta.id.clone());
        create_dir_all(&pfx_dir)?;
        Ok(format!(
            "GAMEID={game_id} WINEPREFIX={} {} {umu:?} {launch}",
            pfx_dir.to_string_lossy(),
            match meta.target_platform {
                Platform::Linux => "UMU_NO_PROTON=1",
                _ => "",
            },
            umu = UMU_LAUNCHER_EXECUTABLE
                .as_ref()
                .expect("Failed to get UMU_LAUNCHER_EXECUTABLE as ref"),
            launch = launch_command,
        ))
    }

    fn valid_for_platform(&self, _db: &Database, _target: &Platform) -> bool {
        let Some(compat_info) = &*COMPAT_INFO else {
            return false;
        };
        compat_info.umu_installed
    }
}

pub struct AsahiMuvmLauncher;
impl ProcessHandler for AsahiMuvmLauncher {
    fn create_launch_process(
        &self,
        meta: &DownloadableMetadata,
        launch_command: String,
        game_version: &GameVersion,
        current_dir: &str,
    ) -> Result<String, ProcessError> {
        let umu_launcher = UMULauncher {};
        let umu_string = umu_launcher.create_launch_process(
            meta,
            launch_command,
            game_version,
            current_dir,
        )?;
        let mut args_cmd = umu_string
            .split("umu-run")
            .collect::<Vec<&str>>()
            .into_iter();
        let args = args_cmd
            .next()
            .ok_or(ProcessError::InvalidArguments(umu_string.clone()))?
            .trim();
        let cmd = format!(
            "umu-run{}",
            args_cmd
                .next()
                .ok_or(ProcessError::InvalidArguments(umu_string.clone()))?
        );

        Ok(format!("{args} muvm -- {cmd}"))
    }

    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    fn valid_for_platform(&self, _db: &Database, _target: &Platform) -> bool {
        #[cfg(not(target_os = "linux"))]
        return false;

        #[cfg(not(target_arch = "aarch64"))]
        return false;

        let page_size = page_size::get();
        if page_size != 16384 {
            return false;
        }

        let Some(compat_info) = &*COMPAT_INFO else {
            return false;
        };

        compat_info.umu_installed
    }
}
