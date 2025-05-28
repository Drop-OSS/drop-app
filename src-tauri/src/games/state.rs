use crate::database::{
    db::borrow_db_checked,
    models::data::{ApplicationTransientStatus, GameDownloadStatus},
};

pub type GameStatusWithTransient = (
    Option<GameDownloadStatus>,
    Option<ApplicationTransientStatus>,
);
pub struct GameStatusManager {}

impl GameStatusManager {
    pub fn fetch_state(game_id: &String) -> GameStatusWithTransient {
        let db_lock = borrow_db_checked();
        let online_state = match db_lock.applications.installed_game_version.get(game_id) {
            Some(meta) => db_lock.applications.transient_statuses.get(meta).cloned(),
            None => None,
        };
        let offline_state = db_lock.applications.game_statuses.get(game_id).cloned();
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
