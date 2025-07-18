use std::{fmt::Display, io::Error};

use serde_with::SerializeDisplay;

#[derive(SerializeDisplay)]
pub enum ProcessError {
    SetupRequired,
    NotInstalled,
    AlreadyRunning,
    NotDownloaded,
    InvalidID,
    InvalidVersion,
    IOError(Error),
    FormatError(String), // String errors supremacy
    InvalidPlatform,
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProcessError::SetupRequired => "Game not set up",
            ProcessError::NotInstalled => "Game not installed",
            ProcessError::AlreadyRunning => "Game already running",
            ProcessError::NotDownloaded => "Game not downloaded",
            ProcessError::InvalidID => "Invalid Game ID",
            ProcessError::InvalidVersion => "Invalid Game version",
            ProcessError::IOError(error) => &error.to_string(),
            ProcessError::InvalidPlatform => "This Game cannot be played on the current platform",
            ProcessError::FormatError(e) => &format!("Failed to format template: {e}"),
        };
        write!(f, "{s}")
    }
}
