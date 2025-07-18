use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use http::StatusCode;
use serde_with::SerializeDisplay;
use url::ParseError;

use super::drop_server_error::DropServerError;

#[derive(Debug, SerializeDisplay)]
pub enum RemoteAccessError {
    FetchError(Arc<reqwest::Error>),
    ParsingError(ParseError),
    InvalidEndpoint,
    HandshakeFailed(String),
    GameNotFound(String),
    InvalidResponse(DropServerError),
    UnparseableResponse(String),
    ManifestDownloadFailed(StatusCode, String),
    OutOfSync,
    Cache(cacache::Error),
}

impl Display for RemoteAccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteAccessError::FetchError(error) => {
                if error.is_connect() {
                    return write!(f, "Failed to connect to Drop server. Check if you access Drop through a browser, and then try again.");
                }

                write!(
                    f,
                    "{}: {}",
                    error,
                    error
                        .source()
                        .map(|e| e.to_string())
                        .or_else(|| Some("Unknown error".to_string()))
                        .unwrap()
                )
            },
            RemoteAccessError::ParsingError(parse_error) => {
                write!(f, "{parse_error}")
            }
            RemoteAccessError::InvalidEndpoint => write!(f, "invalid drop endpoint"),
            RemoteAccessError::HandshakeFailed(message) => write!(f, "failed to complete handshake: {message}"),
            RemoteAccessError::GameNotFound(id) => write!(f, "could not find game on server: {id}"),
            RemoteAccessError::InvalidResponse(error) => write!(f, "server returned an invalid response: {}, {}", error.status_code, error.status_message),
            RemoteAccessError::UnparseableResponse(error) => write!(f, "server returned an invalid response: {}", error),
            RemoteAccessError::ManifestDownloadFailed(status, response) => write!(
                f,
                "failed to download game manifest: {status} {response}"
            ),
            RemoteAccessError::OutOfSync => write!(f, "server's and client's time are out of sync. Please ensure they are within at least 30 seconds of each other"),
            RemoteAccessError::Cache(error) => write!(f, "Cache Error: {error}"),
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
