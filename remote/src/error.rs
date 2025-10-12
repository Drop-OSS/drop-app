use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use http::{HeaderName, StatusCode, header::ToStrError};
use serde_with::SerializeDisplay;
use url::ParseError;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DropServerError {
    pub status_code: usize,
    pub status_message: String,
    // pub message: String,
    // pub url: String,
}

#[derive(Debug, SerializeDisplay)]
pub enum RemoteAccessError {
    FetchError(Arc<reqwest::Error>),
    FetchErrorWS(Arc<reqwest_websocket::Error>),
    ParsingError(ParseError),
    InvalidEndpoint,
    HandshakeFailed(String),
    GameNotFound(String),
    InvalidResponse(DropServerError),
    UnparseableResponse(String),
    ManifestDownloadFailed(StatusCode, String),
    OutOfSync,
    Cache(std::io::Error),
    CorruptedState,
}

impl Display for RemoteAccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteAccessError::FetchError(error) => {
                if error.is_connect() {
                    return write!(
                        f,
                        "Failed to connect to Drop server. Check if you access Drop through a browser, and then try again."
                    );
                }

                write!(
                    f,
                    "{}: {}",
                    error,
                    error
                        .source()
                        .map(std::string::ToString::to_string)
                        .unwrap_or("Unknown error".to_string())
                )
            }
            RemoteAccessError::FetchErrorWS(error) => write!(
                f,
                "{}: {}",
                error,
                error
                    .source()
                    .map(std::string::ToString::to_string)
                    .unwrap_or("Unknown error".to_string())
            ),
            RemoteAccessError::ParsingError(parse_error) => {
                write!(f, "{parse_error}")
            }
            RemoteAccessError::InvalidEndpoint => write!(f, "invalid drop endpoint"),
            RemoteAccessError::HandshakeFailed(message) => {
                write!(f, "failed to complete handshake: {message}")
            }
            RemoteAccessError::GameNotFound(id) => write!(f, "could not find game on server: {id}"),
            RemoteAccessError::InvalidResponse(error) => write!(
                f,
                "server returned an invalid response: {}, {}",
                error.status_code, error.status_message
            ),
            RemoteAccessError::UnparseableResponse(error) => {
                write!(f, "server returned an invalid response: {error}")
            }
            RemoteAccessError::ManifestDownloadFailed(status, response) => {
                write!(f, "failed to download game manifest: {status} {response}")
            }
            RemoteAccessError::OutOfSync => write!(
                f,
                "server's and client's time are out of sync. Please ensure they are within at least 30 seconds of each other"
            ),
            RemoteAccessError::Cache(error) => write!(f, "Cache Error: {error}"),
            RemoteAccessError::CorruptedState => write!(
                f,
                "Drop encountered a corrupted internal state. Please report this to the developers, with details of reproduction."
            ),
        }
    }
}

impl From<reqwest::Error> for RemoteAccessError {
    fn from(err: reqwest::Error) -> Self {
        RemoteAccessError::FetchError(Arc::new(err))
    }
}
impl From<reqwest_websocket::Error> for RemoteAccessError {
    fn from(err: reqwest_websocket::Error) -> Self {
        RemoteAccessError::FetchErrorWS(Arc::new(err))
    }
}
impl From<ParseError> for RemoteAccessError {
    fn from(err: ParseError) -> Self {
        RemoteAccessError::ParsingError(err)
    }
}
impl std::error::Error for RemoteAccessError {}

#[derive(Debug, SerializeDisplay)]
pub enum CacheError {
    HeaderNotFound(HeaderName),
    ParseError(ToStrError),
    Remote(RemoteAccessError),
    ConstructionError(http::Error),
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CacheError::HeaderNotFound(header_name) => {
                format!("Could not find header {header_name} in cache")
            }
            CacheError::ParseError(to_str_error) => {
                format!("Could not parse cache with error {to_str_error}")
            }
            CacheError::Remote(remote_access_error) => {
                format!("Cache got remote access error: {remote_access_error}")
            }
            CacheError::ConstructionError(error) => {
                format!("Could not construct cache body with error {error}")
            }
        };
        write!(f, "{s}")
    }
}
