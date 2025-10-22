use std::collections::HashMap;

use database::models::Game;
use serde::Serialize;

use crate::{app_status::AppStatus, user::User};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    status: AppStatus,
    user: Option<User>,
    games: HashMap<String, Game>,
}
impl AppState {
    pub fn new(status: AppStatus, user: Option<User>, games: HashMap<String, Game>) -> Self {
        Self {
            status,
            user,
            games,
        }
    }

    pub fn status(&self) -> &AppStatus {
        &self.status
    }
    pub fn status_mut(&mut self) -> &mut AppStatus {
        &mut self.status
    }
    pub fn games(&self) -> &HashMap<String, Game> {
        &self.games
    }
    pub fn games_mut(&mut self) -> &mut HashMap<String, Game> {
        &mut self.games
    }
    pub fn user(&self) -> &Option<User> {
        &self.user
    }
    pub fn user_mut(&mut self) -> &mut Option<User> {
        &mut self.user
    }
}
