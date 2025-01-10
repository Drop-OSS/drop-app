use std::sync::{mpsc::SendError, Arc, Mutex};

use crate::{
    download_manager::{download_manager::DownloadManagerSignal, downloadable::Downloadable},
    error::user_error::UserValue,
    AppState,
};

use super::download_agent::GameDownloadAgent;

#[tauri::command]
pub fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> UserValue<(), SendError<DownloadManagerSignal>> {
    let sender = state.lock().unwrap().download_manager.get_sender();
    let game_download_agent = Arc::new(Box::new(GameDownloadAgent::new(
        game_id,
        game_version,
        install_dir,
        sender,
    )) as Box<dyn Downloadable + Send + Sync>);
    state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)
        .into()
}
