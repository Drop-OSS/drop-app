use serde::Serialize;

use crate::{app_status::AppStatus, user::User};

#[derive(Clone, Serialize)]
pub enum UmuState {
    NotNeeded,
    NotInstalled,
    Installed,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub status: AppStatus,
    pub user: Option<User>,
    pub umu_state: UmuState,
}
