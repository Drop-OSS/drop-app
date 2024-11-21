use std::sync::Mutex;

use log::info;

use crate::{AppError, AppState};

#[tauri::command]
pub fn download_game(
    game_id: String,
    game_version: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    
    state
        .lock()
        .unwrap()
        .download_manager
        .queue_game(game_id, game_version, 0)
        .map_err(|_| AppError::Signal)
}

#[tauri::command]
pub fn get_current_game_download_progress(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<f64, AppError> {
    match state
        .lock()
        .unwrap()
        .download_manager
        .get_current_game_download_progress()
        {
            Some(progress) => Ok(progress),
            None => Err(AppError::DoesNotExist),
        }
}

#[tauri::command]
pub fn stop_game_download(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String
) {
    info!("Cancelling game download {}", game_id);
    state
        .lock()
        .unwrap()
        .download_manager
        .cancel_download(game_id);
}
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
