use std::fmt::Display;

use http::{header::ToStrError, HeaderName};
use serde_with::SerializeDisplay;

use crate::error::remote_access_error::RemoteAccessError;

#[derive(Debug, SerializeDisplay)]
pub enum CacheError {
    HeaderNotFound(HeaderName),
    ParseError(ToStrError),
    Remote(RemoteAccessError)
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::HeaderNotFound(header_name) => write!(f, "Could not find header {} in cache", header_name),
            CacheError::ParseError(to_str_error) => write!(f, "Could not parse cache with error {}", to_str_error),
            CacheError::Remote(remote_access_error) => write!(f, "Cache got remote access error: {}", remote_access_error),
        }
    }
}

