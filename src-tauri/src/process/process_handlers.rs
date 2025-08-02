use log::debug;

use crate::{
    AppState,
    database::models::data::{Database, DownloadableMetadata, GameVersion},
    process::process_manager::{Platform, ProcessHandler},
};

pub struct NativeGameLauncher;
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

    fn valid_for_platform(&self, _db: &Database, _state: &AppState, _target: &Platform) -> bool {
        true
    }
}

pub const UMU_LAUNCHER_EXECUTABLE: &str = "umu-run";
pub struct UMULauncher;
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

    fn valid_for_platform(&self, _db: &Database, state: &AppState, _target: &Platform) -> bool {
        let Some(ref compat_info) = state.compat_info else {
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
        args: Vec<String>,
        game_version: &GameVersion,
        current_dir: &str,
    ) -> String {
        let umu_launcher = UMULauncher {};
        let umu_string = umu_launcher.create_launch_process(
            meta,
            launch_command,
            args,
            game_version,
            current_dir,
        );
        let mut args_cmd = umu_string.split("umu-run").collect::<Vec<&str>>().into_iter();
        let args = args_cmd.next().unwrap().trim();
        let cmd = format!("umu-run{}", args_cmd.next().unwrap());

        format!("{} muvm -- {}", args, cmd)
    }

    #[allow(unreachable_code)]
    fn valid_for_platform(&self, _db: &Database, state: &AppState, _target: &Platform) -> bool {
        #[cfg(not(target_os = "linux"))]
        return false;

        #[cfg(not(target_arch = "aarch64"))]
        return false;

        let page_size = page_size::get();
        if page_size != 16384 {
            return false;
        }

        let Some(ref compat_info) = state.compat_info else {
            return false;
        };

        compat_info.umu_installed
    }
}
