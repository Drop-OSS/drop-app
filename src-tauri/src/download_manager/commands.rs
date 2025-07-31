use std::sync::Mutex;

use crate::{database::models::data::DownloadableMetadata, AppState};

#[tauri::command]
pub fn pause_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    state.lock().unwrap().download_manager.pause_downloads();
}

#[tauri::command]
pub fn resume_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    state.lock().unwrap().download_manager.resume_downloads();
}

#[tauri::command]
pub fn move_download_in_queue(
    state: tauri::State<'_, Mutex<AppState>>,
    old_index: usize,
    new_index: usize,
) {
    state
        .lock()
        .unwrap()
        .download_manager
        .rearrange(old_index, new_index);
}

#[tauri::command]
pub fn cancel_game(state: tauri::State<'_, Mutex<AppState>>, meta: DownloadableMetadata) {
    state.lock().unwrap().download_manager.cancel(meta);
}
