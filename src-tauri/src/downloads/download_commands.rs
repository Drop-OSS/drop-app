use std::sync::Mutex;

use crate::AppState;

#[tauri::command]
pub fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    state
        .lock()
        .unwrap()
        .download_manager
        .queue_game(game_id, game_version, install_dir)
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
