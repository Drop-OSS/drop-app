
use crate::{
    db::{GameStatus, GameTransientStatus},
    DB,
};

pub type GameStatusWithTransient = (Option<GameStatus>, Option<GameTransientStatus>);
pub struct GameStatusManager {}

impl GameStatusManager {
    pub fn fetch_state(game_id: &String) -> GameStatusWithTransient {
        let db_lock = DB.borrow_data().unwrap();
        let offline_state = db_lock.games.statuses.get(game_id).cloned();
        let online_state = db_lock.games.transient_statuses.get(game_id).cloned();
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
