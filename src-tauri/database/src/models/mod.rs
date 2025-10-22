mod v1;
mod v2;
mod v3;
mod v4;

use std::{hash::Hash, path::PathBuf};

use native_model::native_model;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::db::DropDatabaseSerializer;


// NOTE: Within each version, you should NEVER use these types.
// Declare it using the actual version that it is from, i.e. v1::Settings rather than just Settings from here

pub type GameVersion = v1::GameVersion;
pub type Database = v4::Database;
pub type Settings = v1::Settings;
pub type DatabaseAuth = v1::DatabaseAuth;

pub type GameDownloadStatus = v2::GameDownloadStatus;
pub type ApplicationTransientStatus = v1::ApplicationTransientStatus;
/**
 * Need to be universally accessible by the ID, and the version is just a couple sprinkles on top
 */
pub type DownloadableMetadata = v1::DownloadableMetadata;
pub type DownloadType = v1::DownloadType;
pub type DatabaseApplications = v2::DatabaseApplications;
// pub type DatabaseCompatInfo = v2::DatabaseCompatInfo;

pub type Game = v1::Game;

impl Game {
    pub fn id(&self) -> &String {
        &self.id
    }
}

pub type DatabaseInterface =
    rustbreak::Database<Database, rustbreak::backend::PathBackend, DropDatabaseSerializer>;

pub type LibraryMetadata = v1::LibraryMetadata;
pub type LibraryProviderMetadata = v1::LibraryProviderMetadata;
pub type ProviderType = v1::ProviderType;

pub type Collection = v1::Collection;
pub type CollectionObject = v1::CollectionObject;

impl PartialEq for DownloadableMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.download_type == other.download_type
    }
}
impl Hash for DownloadableMetadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.download_type.hash(state);
    }
}
impl LibraryProviderMetadata {
    pub fn provider(&self) -> &ProviderType {
        &self.provider
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl Database {
    pub fn new<T: Into<PathBuf>>(
        games_base_dir: T,
        prev_database: Option<PathBuf>,
        cache_dir: PathBuf,
    ) -> Self {
        Self {
            prev_database,
            settings: Settings::default(),
            cache_dir,
            compat_info: None,
            library: v1::LibraryMetadata { providers: vec![] },
        }
    }
}
impl DatabaseAuth {
    pub fn new(
        private: String,
        cert: String,
        client_id: String,
        web_token: Option<String>,
    ) -> Self {
        Self {
            private,
            cert,
            client_id,
            web_token,
        }
    }
}

impl LibraryMetadata {
    pub fn providers(&self) -> &Vec<LibraryProviderMetadata> {
        &self.providers
    }
}