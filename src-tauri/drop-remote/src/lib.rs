use serde::{Deserialize, Serialize};
use url::Url;

pub mod auth;
pub mod cache;
pub mod fetch_object;
pub mod requests;
pub mod utils;

#[derive(Serialize, Deserialize, Clone)]
struct DropRemoteAuth {
    private: String,
    cert: String,
    client_id: String,
    web_token: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DropRemoteContext {
    base_url: Url,
    auth: Option<DropRemoteAuth>,
}


impl DropRemoteContext {
    pub fn new(base_url: Url) -> Self {
        DropRemoteContext { base_url, auth: None }
    }
}