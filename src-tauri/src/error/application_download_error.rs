use std::{
    fmt::{Display, Formatter},
    io, sync::Arc,
};

use serde_with::SerializeDisplay;
use humansize::{format_size, BINARY};

use super::remote_access_error::RemoteAccessError;

// TODO: Rename / separate from downloads
#[derive(Debug, SerializeDisplay)]
pub enum ApplicationDownloadError {
    NotInitialized,
    Communication(RemoteAccessError),
    DiskFull(u64, u64),
    #[allow(dead_code)]
    Checksum,
    Lock,
    IoError(Arc<io::Error>),
    DownloadError,
}

impl Display for ApplicationDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationDownloadError::NotInitialized => write!(f, "Download not initalized, did something go wrong?"),
            ApplicationDownloadError::DiskFull(required, available) => write!(
                f,
                "Game requires {}, {} remaining left on disk.",
                format_size(*required, BINARY),
                format_size(*available, BINARY),
            ),
            ApplicationDownloadError::Communication(error) => write!(f, "{error}"),
            ApplicationDownloadError::Lock => write!(
                f,
                "failed to acquire lock. Something has gone very wrong internally. Please restart the application"
            ),
            ApplicationDownloadError::Checksum => {
                write!(f, "checksum failed to validate for download")
            }
            ApplicationDownloadError::IoError(error) => write!(f, "io error: {error}"),
            ApplicationDownloadError::DownloadError => write!(
                f,
                "Download failed. See Download Manager status for specific error"
            ),
        }
    }
}
