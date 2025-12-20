use database::models::data::{
    ApplicationTransientStatus, Database, DownloadType, GameDownloadStatus,
};

pub type GameStatusWithTransient = (
    Option<GameDownloadStatus>,
    Option<ApplicationTransientStatus>,
);
pub struct GameStatusManager {}

impl GameStatusManager {
    pub fn fetch_state(game_id: &String, database: &Database) -> GameStatusWithTransient {
        let online_state = database
            .applications
            .transient_statuses
            .iter()
            .find(|v| v.0.id == *game_id && v.0.download_type == DownloadType::Game)
            .map(|v| v.1.clone())
            .clone();

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
