use std::{
    fmt::{Display, Formatter},
    io,
};

use serde_with::SerializeDisplay;
use human_size::{SpecificSize, Byte};

use super::remote_access_error::RemoteAccessError;

// TODO: Rename / separate from downloads
#[derive(Debug, SerializeDisplay)]
pub enum ApplicationDownloadError {
    Communication(RemoteAccessError),
    DiskFull(usize, usize),
    Checksum,
    Lock,
    IoError(io::ErrorKind),
    DownloadError,
}

impl Display for ApplicationDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationDownloadError::DiskFull(required, remaining) => write!(
                f,
                "requires {}, {} remaining left on disk",
                SpecificSize::new::<f64>(*required as f64, Byte).unwrap().to_string(),
                SpecificSize::new::<f64>(*remaining as f64, Byte).unwrap().to_string(),
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
                "download failed. See Download Manager status for specific error"
            ),
        }
    }
}
