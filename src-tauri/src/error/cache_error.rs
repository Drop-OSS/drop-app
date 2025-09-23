use std::fmt::Display;

use http::{header::ToStrError, HeaderName};
use serde_with::SerializeDisplay;

use crate::error::remote_access_error::RemoteAccessError;

#[derive(Debug, SerializeDisplay)]
pub enum CacheError {
    HeaderNotFound(HeaderName),
    ParseError(ToStrError),
    Remote(RemoteAccessError),
    ConstructionError(http::Error)
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CacheError::HeaderNotFound(header_name) => format!("Could not find header {header_name} in cache"),
            CacheError::ParseError(to_str_error) => format!("Could not parse cache with error {to_str_error}"),
            CacheError::Remote(remote_access_error) => format!("Cache got remote access error: {remote_access_error}"),
            CacheError::ConstructionError(error) => format!("Could not construct cache body with error {error}"),
        };
        write!(f, "{s}")
    }
}