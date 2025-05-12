use std::{collections::HashMap, path::PathBuf, str::FromStr};

use log::warn;

use crate::{database::db::GameVersion, error::backup_error::BackupError, process::process_manager::Platform};

use super::path::CommonPath;

pub struct BackupManager<'a> {
    pub current_platform: Platform,
    pub sources: HashMap<(Platform, Platform), &'a (dyn BackupHandler + Sync + Send)>,
}

impl BackupManager<'_> {
    pub fn new() -> Self {
        BackupManager {
            #[cfg(target_os = "windows")]
            current_platform: Platform::Windows,

            #[cfg(target_os = "macos")]
            current_platform: Platform::MacOs,

            #[cfg(target_os = "linux")]
            current_platform: Platform::Linux,

            sources: HashMap::from([
                // Current platform to target platform
                (
                    (Platform::Windows, Platform::Windows),
                    &WindowsBackupManager {} as &(dyn BackupHandler + Sync + Send),
                ),
                (
                    (Platform::Linux, Platform::Linux),
                    &LinuxBackupManager {} as &(dyn BackupHandler + Sync + Send),
                ),
                (
                    (Platform::MacOs, Platform::MacOs),
                    &MacBackupManager {} as &(dyn BackupHandler + Sync + Send),
                ),
                
            ]),
        }
    }
    
}

pub trait BackupHandler: Send + Sync {
    fn root_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError>;
    fn game_translate(&self, _path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> { Ok(PathBuf::from_str(&game.game_id).unwrap()) }
    fn base_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> { Ok(self.root_translate(path, game)?.join(self.game_translate(path, game)?)) }
    fn home_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { let c = CommonPath::Home.get().ok_or(BackupError::NotFound); println!("{:?}", c); c }
    fn store_user_id_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError>;
    fn os_user_name_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { Ok(PathBuf::from_str(&whoami::username()).unwrap()) }
    fn win_app_data_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winAppData>"); Err(BackupError::InvalidSystem) }
    fn win_local_app_data_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winLocalAppData>"); Err(BackupError::InvalidSystem) }
    fn win_local_app_data_low_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winLocalAppDataLow>"); Err(BackupError::InvalidSystem) }
    fn win_documents_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winDocuments>"); Err(BackupError::InvalidSystem) }
    fn win_public_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winPublic>"); Err(BackupError::InvalidSystem) }
    fn win_program_data_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winProgramData>"); Err(BackupError::InvalidSystem) }
    fn win_dir_translate(&self, _path: &PathBuf,_game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected Windows Reference in Backup <winDir>"); Err(BackupError::InvalidSystem) }
    fn xdg_data_translate(&self, _path: &PathBuf,_game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected XDG Reference in Backup <xdgData>"); Err(BackupError::InvalidSystem) }
    fn xdg_config_translate(&self, _path: &PathBuf,_game: &GameVersion) -> Result<PathBuf, BackupError> { warn!("Unexpected XDG Reference in Backup <xdgConfig>"); Err(BackupError::InvalidSystem) }
    fn skip_translate(&self, _path: &PathBuf, _game: &GameVersion) -> Result<PathBuf, BackupError> { Ok(PathBuf::new()) }
}

pub struct LinuxBackupManager {}
impl BackupHandler for LinuxBackupManager {
    fn root_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> {
        println!("Root translate");
        PathBuf::from_str("~").map_err(|_| BackupError::ParseError)
    }

    fn store_user_id_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> {
        println!("Store user id translate");
        PathBuf::from_str("ID").map_err(|_| BackupError::ParseError)
    }
}
pub struct WindowsBackupManager {}
impl BackupHandler for WindowsBackupManager {
    fn root_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> {
        todo!()
    }

    fn store_user_id_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> {
        todo!()
    }
}
pub struct MacBackupManager {}
impl BackupHandler for MacBackupManager {
    fn root_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> {
        todo!()
    }

    fn store_user_id_translate(&self, path: &PathBuf, game: &GameVersion) -> Result<PathBuf, BackupError> {
        todo!()
    }
}