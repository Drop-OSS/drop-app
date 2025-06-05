use std::sync::Mutex;

use crate::{
    database::models::data::{DownloadType, DownloadableMetadata},
    download_manager::download_manager::QueueMetadata,
    AppState,
};

#[tauri::command]
pub fn pause_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    state.lock().unwrap().download_manager.pause_downloads()
}

#[tauri::command]
pub fn resume_downloads(state: tauri::State<'_, Mutex<AppState>>) {
    state.lock().unwrap().download_manager.resume_downloads()
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
        .rearrange(old_index, new_index)
}

#[tauri::command]
pub fn cancel_game(state: tauri::State<'_, Mutex<AppState>>, meta: DownloadableMetadata) {
    state.lock().unwrap().download_manager.cancel(meta)
}

#[tauri::command]
pub fn get_queue_metadata(
    state: tauri::State<'_, Mutex<AppState>>,
    meta: DownloadableMetadata,
) -> Option<QueueMetadata> {
    match meta.download_type {
        DownloadType::Game => {
            let state = state.lock().unwrap();
            let game = state.games.get(&meta.id).unwrap();
            Some(QueueMetadata {
                cover: game.m_cover_object_id.clone(),
                m_short_description: game.m_short_description.clone(),
                m_name: game.m_name.clone(),
            })
        }
        DownloadType::Tool => Some(QueueMetadata {
            cover: "IDK Man".to_string(),
            m_short_description: "This is a tool".to_string(),
            m_name: "Download".to_string(),
        }),
        DownloadType::DLC => unimplemented!(),
        DownloadType::Mod => unimplemented!(),
    }
}
