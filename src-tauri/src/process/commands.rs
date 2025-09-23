use std::sync::Mutex;

use crate::{error::process_error::ProcessError, lock, AppState};

#[tauri::command]
pub fn launch_game(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = lock!(state);
    let mut process_manager_lock = lock!(state_lock.process_manager);

    //let meta = DownloadableMetadata {
    //    id,
    //    version: Some(version),
    //    download_type: DownloadType::Game,
    //};

    match process_manager_lock.launch_process(id, &state_lock) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    drop(process_manager_lock);
    drop(state_lock);

    Ok(())
}

#[tauri::command]
pub fn kill_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = lock!(state);
    let mut process_manager_lock = lock!(state_lock.process_manager);
    process_manager_lock
        .kill_game(game_id)
        .map_err(ProcessError::IOError)
}

#[tauri::command]
pub fn open_process_logs(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = lock!(state);
    let mut process_manager_lock = lock!(state_lock.process_manager);
    process_manager_lock.open_process_logs(game_id)
}
