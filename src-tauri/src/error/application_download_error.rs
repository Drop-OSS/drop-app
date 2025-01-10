use std::{
    fmt::{Display, Formatter},
    io,
};

use super::remote_access_error::RemoteAccessError;

// TODO: Rename / separate from downloads
#[derive(Debug, Clone)]
pub enum ApplicationDownloadError {
    Communication(RemoteAccessError),
    Checksum,
    Setup(SetupError),
    Lock,
    IoError(io::ErrorKind),
    DownloadError,
}

impl Display for ApplicationDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationDownloadError::Communication(error) => write!(f, "{}", error),
            ApplicationDownloadError::Setup(error) => write!(f, "An error occurred while setting up the download: {}", error),
            ApplicationDownloadError::Lock => write!(f, "Failed to acquire lock. Something has gone very wrong internally. Please restart the application"),
            ApplicationDownloadError::Checksum => write!(f, "Checksum failed to validate for download"),
            ApplicationDownloadError::IoError(error) => write!(f, "{}", error),
            ApplicationDownloadError::DownloadError => write!(f, "Download failed. See Download Manager status for specific error"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SetupError {
    Context,
}

impl Display for SetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupError::Context => write!(f, "Failed to generate contexts for download"),
        }
    }
}
