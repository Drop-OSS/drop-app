use std::sync::nonpoison::Mutex;

use process::{PROCESS_MANAGER, error::ProcessError};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::AppState;

#[tauri::command]
pub fn launch_game(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock();
    let mut process_manager_lock = PROCESS_MANAGER.lock();
    //let meta = DownloadableMetadata {
    //    id,
    //    version: Some(version),
    //    download_type: DownloadType::Game,
    //};

    match process_manager_lock.launch_process(id) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    drop(process_manager_lock);
    drop(state_lock);

    Ok(())
}

#[tauri::command]
pub fn kill_game(game_id: String) -> Result<(), ProcessError> {
    PROCESS_MANAGER
        .lock()
        .kill_game(game_id)
        .map_err(ProcessError::IOError)
}

#[tauri::command]
pub fn open_process_logs(game_id: String, app_handle: AppHandle) -> Result<(), ProcessError> {
    let process_manager_lock = PROCESS_MANAGER.lock();

    let dir = process_manager_lock.get_log_dir(game_id);
    app_handle
        .opener()
        .open_path(dir.display().to_string(), None::<&str>)
        .map_err(ProcessError::OpenerError)
}
