use std::fmt::Display;

use database::models::LibraryProviderMetadata;
use remote::error::RemoteAccessError;
use serde_with::SerializeDisplay;

#[derive(Debug, SerializeDisplay)]
pub enum LibraryError {
    ProviderConnection(ProviderError),
    FetchError(RemoteAccessError)
}
#[derive(Debug, SerializeDisplay)]
pub struct ProviderError {
    provider: LibraryProviderMetadata
}

impl Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl From<RemoteAccessError> for LibraryError {
    fn from(value: RemoteAccessError) -> Self {
        LibraryError::FetchError(value)
    }
}