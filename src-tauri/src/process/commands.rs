use tokio::sync::Mutex;

use crate::{AppState, error::process_error::ProcessError};

#[tauri::command]
pub async fn launch_game(
    id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().await;
    let mut process_manager_lock = state_lock.process_manager.lock().await;

    //let meta = DownloadableMetadata {
    //    id,
    //    version: Some(version),
    //    download_type: DownloadType::Game,
    //};

    match process_manager_lock.launch_process(id).await {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    drop(process_manager_lock);
    drop(state_lock);

    Ok(())
}

#[tauri::command]
pub async fn kill_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().await;
    let mut process_manager_lock = state_lock.process_manager.lock().await;
    process_manager_lock
        .kill_game(game_id)
        .map_err(ProcessError::IOError)
}

#[tauri::command]
pub async fn open_process_logs(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().await;
    let mut process_manager_lock = state_lock.process_manager.lock().await;
    process_manager_lock.open_process_logs(game_id)
}
