use std::{
    fmt::{Display, Formatter},
    io,
};

use serde_with::SerializeDisplay;

use super::{remote_access_error::RemoteAccessError, setup_error::SetupError};

// TODO: Rename / separate from downloads
#[derive(Debug, SerializeDisplay)]
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
            ApplicationDownloadError::Setup(error) => write!(f, "an error occurred while setting up the download: {}", error),
            ApplicationDownloadError::Lock => write!(f, "failed to acquire lock. Something has gone very wrong internally. Please restart the application"),
            ApplicationDownloadError::Checksum => write!(f, "checksum failed to validate for download"),
            ApplicationDownloadError::IoError(error) => write!(f, "{}", error),
            ApplicationDownloadError::DownloadError => write!(f, "download failed. See Download Manager status for specific error"),
        }
    }
}
