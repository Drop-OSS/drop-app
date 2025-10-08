use std::{path::PathBuf, sync::LazyLock};

pub enum CommonPath {
    Config,
    Data,
    DataLocal,
    DataLocalLow,
    Document,
    Home,
    Public,
    SavedGames,
}

impl CommonPath {
    pub fn get(&self) -> Option<PathBuf> {
        static CONFIG: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::config_dir());
        static DATA: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::data_dir());
        static DATA_LOCAL: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::data_local_dir());
        static DOCUMENT: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::document_dir());
        static HOME: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::home_dir());
        static PUBLIC: LazyLock<Option<PathBuf>> = LazyLock::new(|| dirs::public_dir());

        #[cfg(windows)]
        static DATA_LOCAL_LOW: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
            known_folders::get_known_folder_path(known_folders::KnownFolder::LocalAppDataLow)
        });
        #[cfg(not(windows))]
        static DATA_LOCAL_LOW: Option<PathBuf> = None;

        #[cfg(windows)]
        static SAVED_GAMES: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
            known_folders::get_known_folder_path(known_folders::KnownFolder::SavedGames)
        });
        #[cfg(not(windows))]
        static SAVED_GAMES: Option<PathBuf> = None;

        match self {
            Self::Config => CONFIG.clone(),
            Self::Data => DATA.clone(),
            Self::DataLocal => DATA_LOCAL.clone(),
            Self::DataLocalLow => DATA_LOCAL_LOW.clone(),
            Self::Document => DOCUMENT.clone(),
            Self::Home => HOME.clone(),
            Self::Public => PUBLIC.clone(),
            Self::SavedGames => SAVED_GAMES.clone(),
        }
    }
}
