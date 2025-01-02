
use std::sync::Arc;

use crate::{
    db::{ApplicationStatus, ApplicationTransientStatus}, download_manager::downloadable_metadata::DownloadableMetadata, DB
};

pub type ApplicationStatusWithTransient = (Option<ApplicationStatus>, Option<ApplicationTransientStatus>);
pub struct DownloadStatusManager {}

impl DownloadStatusManager {
    pub fn fetch_state(id: &DownloadableMetadata) -> ApplicationStatusWithTransient {
        let db_lock = DB.borrow_data().unwrap();
        let offline_state = db_lock.applications.statuses.get(id).cloned();
        let online_state = db_lock.applications.transient_statuses.get(id).cloned();
        drop(db_lock);

        if online_state.is_some() {
            return (None, online_state);
        }

        if offline_state.is_some() {
            return (offline_state, None);
        }

        (None, None)
    }
}
