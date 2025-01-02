use std::sync::{Arc, Mutex};

use crate::{download_manager::{downloadable::Downloadable, downloadable_metadata::DownloadableMetadata}, AppState};

use super::download_agent::GameDownloadAgent;

#[tauri::command]
pub fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let sender = state.lock().unwrap().download_manager.get_sender();
    let game_download_agent = Arc::new(
        Box::new(GameDownloadAgent::new(game_id, game_version, install_dir, sender)) as Box<dyn Downloadable + Send + Sync>
    );
    state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)
        .map_err(|_| "An error occurred while communicating with the download manager.".to_string())
}

#[tauri::command]
pub fn pause_game_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    state.lock().unwrap().download_manager.pause_downloads()
}

#[tauri::command]
pub fn resume_game_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    state.lock().unwrap().download_manager.resume_downloads()
}

#[tauri::command]
pub fn move_game_in_queue(
    state: tauri::State<'_, Mutex<AppState>>,
    old_index: usize,
    new_index: usize,
) {
    state
        .lock()
        .unwrap()
        .download_manager
        .rearrange(old_index, new_index)
}

#[tauri::command]
pub fn cancel_game(state: tauri::State<'_, Mutex<AppState>>, game_id: DownloadableMetadata) {
    state.lock().unwrap().download_manager.cancel(Arc::new(game_id))
}

/*
#[tauri::command]
pub fn get_current_write_speed(state: tauri::State<'_, Mutex<AppState>>) {}
*/

/*
fn use_download_agent(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<Arc<GameDownloadAgent>, String> {
    let lock = state.lock().unwrap();
    let download_agent = lock.download_manager.get(&game_id).ok_or("Invalid game ID")?;
    Ok(download_agent.clone()) // Clones the Arc, not the underlying data structure
}
*/
