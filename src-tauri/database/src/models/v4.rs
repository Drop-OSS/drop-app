use std::path::PathBuf;

use native_model::native_model;
use serde::{Deserialize, Serialize};

use crate::models::{
    v1::{self, LibraryMetadata},
    v2, v3,
};

#[native_model(id = 1, version = 4, with = native_model::rmp_serde_1_3::RmpSerde, from = v3::Database)]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Database {
    #[serde(default)]
    pub settings: v1::Settings,
    #[serde(skip)]
    pub prev_database: Option<PathBuf>,
    pub cache_dir: PathBuf,
    pub compat_info: Option<v2::DatabaseCompatInfo>,
    pub library: v1::LibraryMetadata,
}

impl From<v3::Database> for Database {
    fn from(value: v3::Database) -> Self {
        Self {
            settings: value.settings,
            prev_database: value.prev_database,
            cache_dir: value.cache_dir,
            compat_info: value.compat_info,
            library: v1::LibraryMetadata {
                providers: if let Some(auth) = value.auth {
                    vec![v1::LibraryProviderMetadata {
                        id: 0,
                        name: String::from("Default"),
                        provider: v1::ProviderType::Drop(auth),
                    }]
                } else {
                    vec![]
                },
            },
        }
    }
}
