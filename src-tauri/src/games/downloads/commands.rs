use std::sync::{Arc, Mutex};

use crate::{
    download_manager::{
        download_manager::DownloadManagerSignal, downloadable::Downloadable,
        internal_error::InternalError,
    },
    AppState,
};

use super::download_agent::GameDownloadAgent;

#[tauri::command]
pub fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), InternalError<DownloadManagerSignal>> {
    let sender = state.lock().unwrap().download_manager.get_sender();
    let game_download_agent = Arc::new(Box::new(GameDownloadAgent::new(
        game_id,
        game_version,
        install_dir,
        sender,
    )) as Box<dyn Downloadable + Send + Sync>);
    Ok(state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)?)
}
