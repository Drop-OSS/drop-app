use crate::database::{
    db::borrow_db_checked,
    models::data::{ApplicationTransientStatus, Database, GameDownloadStatus},
};

pub type GameStatusWithTransient = (
    Option<GameDownloadStatus>,
    Option<ApplicationTransientStatus>,
);
pub struct GameStatusManager {}

impl GameStatusManager {
    pub fn fetch_state(game_id: &String, database: &Database) -> GameStatusWithTransient {
        let online_state = match database.applications.installed_game_version.get(game_id) {
            Some(meta) => database.applications.transient_statuses.get(meta).cloned(),
            None => None,
        };
        let offline_state = database.applications.game_statuses.get(game_id).cloned();

        if online_state.is_some() {
            return (None, online_state);
        }

        if offline_state.is_some() {
            return (offline_state, None);
        }

        (None, None)
    }
}
