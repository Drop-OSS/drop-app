use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use http::StatusCode;
use url::ParseError;

use super::drop_server_error::DropServerError;

#[derive(Debug, Clone)]
pub enum RemoteAccessError {
    FetchError(Arc<reqwest::Error>),
    ParsingError(ParseError),
    InvalidEndpoint,
    HandshakeFailed(String),
    GameNotFound,
    InvalidResponse(DropServerError),
    InvalidRedirect,
    ManifestDownloadFailed(StatusCode, String),
    OutOfSync,
    Generic(String),
}

impl Display for RemoteAccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteAccessError::FetchError(error) => write!(
                f,
                "{}: {}",
                error,
                error
                    .source()
                    .map(|e| e.to_string())
                    .or_else(|| Some("Unknown error".to_string()))
                    .unwrap()
            ),
            RemoteAccessError::ParsingError(parse_error) => {
                write!(f, "{}", parse_error)
            }
            RemoteAccessError::InvalidEndpoint => write!(f, "Invalid drop endpoint"),
            RemoteAccessError::HandshakeFailed(message) => write!(f, "Failed to complete handshake: {}", message),
            RemoteAccessError::GameNotFound => write!(f, "Could not find game on server"),
            RemoteAccessError::InvalidResponse(error) => write!(f, "Server returned an invalid response: {} {}", error.status_code, error.status_message),
            RemoteAccessError::InvalidRedirect => write!(f, "Server redirect was invalid"),
            RemoteAccessError::ManifestDownloadFailed(status, response) => write!(
                f,
                "Failed to download game manifest: {} {}",
                status, response
            ),
            RemoteAccessError::OutOfSync => write!(f, "Server's and client's time are out of sync. Please ensure they are within at least 30 seconds of each other."),
            RemoteAccessError::Generic(message) => write!(f, "{}", message),
        }
    }
}

impl From<reqwest::Error> for RemoteAccessError {
    fn from(err: reqwest::Error) -> Self {
        RemoteAccessError::FetchError(Arc::new(err))
    }
}
impl From<ParseError> for RemoteAccessError {
    fn from(err: ParseError) -> Self {
        RemoteAccessError::ParsingError(err)
    }
}
impl std::error::Error for RemoteAccessError {}
