use std::{fmt::Display, io, sync::mpsc::SendError};

use serde_with::SerializeDisplay;

#[derive(SerializeDisplay)]
pub enum DownloadManagerError<T> {
    IOError(io::Error),
    SignalError(SendError<T>),
}
impl<T> Display for DownloadManagerError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadManagerError::IOError(error) => write!(f, "{}", error),
            DownloadManagerError::SignalError(send_error) => write!(f, "{}", send_error),
        }
    }
}
impl<T> From<SendError<T>> for DownloadManagerError<T> {
    fn from(value: SendError<T>) -> Self {
        DownloadManagerError::SignalError(value)
    }
}
impl<T> From<io::Error> for DownloadManagerError<T> {
    fn from(value: io::Error) -> Self {
        DownloadManagerError::IOError(value)
    }
}
