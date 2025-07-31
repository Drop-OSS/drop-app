use std::{fmt::Display, io::Error};

use serde_with::SerializeDisplay;

#[derive(SerializeDisplay)]
pub enum ProcessError {
    NotInstalled,
    AlreadyRunning,
    NotDownloaded,
    InvalidID,
    InvalidVersion,
    IOError(Error),
    FormatError(String), // String errors supremacy
    InvalidPlatform,
    OpenerError(tauri_plugin_opener::Error)
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProcessError::SetupRequired => "Game not set up",
            ProcessError::NotInstalled => "Game not installed",
            ProcessError::AlreadyRunning => "Game already running",
            ProcessError::NotDownloaded => "Game not downloaded",
            ProcessError::InvalidID => "Invalid game ID",
            ProcessError::InvalidVersion => "Invalid game version",
            ProcessError::IOError(error) => &error.to_string(),
            ProcessError::InvalidPlatform => "This game cannot be played on the current platform",
            ProcessError::FormatError(e) => &format!("Failed to format template: {e}"),
            ProcessError::OpenerError(error) => &format!("Failed to open directory: {error}"),
                    };
        write!(f, "{s}")
    }
}
