use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use http::Request;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::UriSchemeResponder;

use crate::{
    errors::DropLibraryError,
    game::{LibraryGame, LibraryGamePreview},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct LibraryProviderIdentifier {
    internal_id: usize,
    name: String,
}

impl PartialEq for LibraryProviderIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

impl Eq for LibraryProviderIdentifier {}

impl Hash for LibraryProviderIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.internal_id.hash(state);
    }
}

impl Display for LibraryProviderIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl LibraryProviderIdentifier {
    pub fn str_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

pub struct LibraryFetchConfig {
    pub hard_refresh: bool,
}

pub trait DropLibraryProvider: Serialize + DeserializeOwned + Sized {
    fn build(identifier: LibraryProviderIdentifier) -> Self;
    fn id(&self) -> &LibraryProviderIdentifier;
    fn load_object(
        &self,
        request: Request<Vec<u8>>,
        responder: UriSchemeResponder,
    ) -> impl Future<Output = Result<(), DropLibraryError>> + Send;

    fn fetch_library(
        &self,
        config: &LibraryFetchConfig,
    ) -> impl Future<Output = Result<Vec<LibraryGamePreview>, DropLibraryError>> + Send;
    fn fetch_game(
        &self,
        config: &LibraryFetchConfig,
    ) -> impl Future<Output = Result<LibraryGame, DropLibraryError>> + Send;

    

    fn owns_game(&self, id: &LibraryProviderIdentifier) -> bool {
        self.id().internal_id == id.internal_id
    }
}
