
use std::sync::Arc;

use crate::{
    db::{ApplicationStatus, ApplicationTransientStatus}, download_manager::downloadable_metadata::DownloadableMetadata, DB
};

pub type GameStatusWithTransient = (Option<ApplicationStatus>, Option<ApplicationTransientStatus>);
pub struct GameStatusManager {}

impl DownloadStatusManager {
    pub fn fetch_state(id: &DownloadableMetadata) -> ApplicationStatusWithTransient {
        let db_lock = DB.borrow_data().unwrap();
        GameStatusManager::fetch_state_with_db(game_id, &db_lock)
    }
    pub fn fetch_state_with_db(
        game_id: &String,
        db_lock: &Database,
    ) -> GameStatusWithTransient {
        let offline_state = db_lock.applications.statuses.get(game_id).cloned();
        let online_state = db_lock.applications.transient_statuses.get(game_id).cloned();

        if online_state.is_some() {
            return (None, online_state);
        }

        if offline_state.is_some() {
            return (offline_state, None);
        }

        (None, None)
    }
}
