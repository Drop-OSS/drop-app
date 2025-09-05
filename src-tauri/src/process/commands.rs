use std::sync::Mutex;

use crate::{error::process_error::ProcessError, AppState};

#[tauri::command]
pub fn launch_game(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();

    //let meta = DownloadableMetadata {
    //    id,
    //    version: Some(version),
    //    download_type: DownloadType::Game,
    //};

    match process_manager_lock.launch_process(id.clone(), &state_lock) {
        Ok(()) => {
            // Start playtime tracking after successful launch
            drop(process_manager_lock);
            
            let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
            if let Err(e) = playtime_manager_lock.start_session(id.clone()) {
                log::warn!("Failed to start playtime tracking for {}: {}", id, e);
            } else {
                log::debug!("Started playtime tracking for game: {}", id);
                crate::playtime::events::push_session_start(&state_lock.app_handle, &id);
            }
            drop(playtime_manager_lock);
        }
        Err(e) => {
            drop(process_manager_lock);
            drop(state_lock);
            return Err(e);
        }
    }

    drop(state_lock);
    Ok(())
}

#[tauri::command]
pub fn kill_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    
    // End playtime tracking before killing the game
    drop(process_manager_lock);
    let playtime_manager_lock = state_lock.playtime_manager.lock().unwrap();
    if let Ok(stats) = playtime_manager_lock.end_session(game_id.clone()) {
        log::debug!("Ended playtime tracking for game: {} (manual kill)", game_id);
        crate::playtime::events::push_session_end(&state_lock.app_handle, &game_id, &stats);
        crate::playtime::events::push_playtime_update(&state_lock.app_handle, &game_id, stats, false);
    }
    drop(playtime_manager_lock);
    
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    process_manager_lock
        .kill_game(game_id)
        .map_err(ProcessError::IOError)
}

#[tauri::command]
pub fn open_process_logs(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    process_manager_lock.open_process_logs(game_id)
}
