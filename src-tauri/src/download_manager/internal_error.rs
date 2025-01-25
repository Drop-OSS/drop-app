use std::{fmt::Display, io, sync::mpsc::SendError};

use serde_with::SerializeDisplay;

#[derive(SerializeDisplay)]
pub enum InternalError<T> {
    IOError(io::Error),
    SignalError(SendError<T>),
}
impl<T> Display for InternalError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalError::IOError(error) => write!(f, "{}", error),
            InternalError::SignalError(send_error) => write!(f, "{}", send_error),
        }
    }
}
impl<T> From<SendError<T>> for InternalError<T> {
    fn from(value: SendError<T>) -> Self {
        InternalError::SignalError(value)
    }
}
impl<T> From<io::Error> for InternalError<T> {
    fn from(value: io::Error) -> Self {
        InternalError::IOError(value)
    }
}
