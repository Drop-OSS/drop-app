use database::models::data::{
    ApplicationTransientStatus, Database, DownloadType, DownloadableMetadata, GameDownloadStatus,
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
            .get(&DownloadableMetadata {
                id: game_id.to_string(),
                download_type: DownloadType::Game,
                version: None,
            })
            .cloned();

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
