use std::sync::Mutex;

use crate::{AppState, database::models::data::DownloadableMetadata, lock};

#[tauri::command]
pub fn pause_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    lock!(state).download_manager.pause_downloads();
}

#[tauri::command]
pub fn resume_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    lock!(state).download_manager.resume_downloads();
}

#[tauri::command]
pub fn move_download_in_queue(
    state: tauri::State<'_, Mutex<AppState>>,
    old_index: usize,
    new_index: usize,
) {
    lock!(state)
        .download_manager
        .rearrange(old_index, new_index);
}

#[tauri::command]
pub fn cancel_game(state: tauri::State<'_, Mutex<AppState>>, meta: DownloadableMetadata) {
    lock!(state).download_manager.cancel(meta);
}
