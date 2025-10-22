use std::{collections::HashMap, path::PathBuf};

use serde_with::serde_as;

use super::{Deserialize, Serialize, native_model, v1, v2};
#[native_model(id = 1, version = 3, with = native_model::rmp_serde_1_3::RmpSerde, from = v2::Database)]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Database {
    #[serde(default)]
    pub settings: v1::Settings,
    pub auth: Option<v1::DatabaseAuth>,
    pub base_url: String,
    pub applications: v2::DatabaseApplications,
    #[serde(skip)]
    pub prev_database: Option<PathBuf>,
    pub cache_dir: PathBuf,
    pub compat_info: Option<v2::DatabaseCompatInfo>,
}

impl From<v2::Database> for Database {
    fn from(value: v2::Database) -> Self {
        Self {
            settings: value.settings,
            auth: value.auth,
            base_url: value.base_url,
            applications: value.applications.into(),
            prev_database: value.prev_database,
            cache_dir: value.cache_dir,
            compat_info: None,
        }
    }
}