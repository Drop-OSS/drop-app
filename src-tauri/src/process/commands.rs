use std::sync::Mutex;

use crate::{
    database::db::GameDownloadStatus,
    download_manager::downloadable_metadata::{DownloadType, DownloadableMetadata},
    error::{process_error::ProcessError, user_error::UserValue},
    games::library::get_current_meta,
    AppState, DB,
};

#[tauri::command]
pub fn launch_game(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> UserValue<(), ProcessError> {
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();

    let version = match DB
        .borrow_data()
        .unwrap()
        .applications
        .game_statuses
        .get(&id)
        .cloned()
    {
        Some(GameDownloadStatus::Installed { version_name, .. }) => version_name,
        Some(GameDownloadStatus::SetupRequired { .. }) => {
            return Err(ProcessError::SetupRequired).into()
        }
        _ => return Err(ProcessError::NotInstalled).into(),
    };

    let meta = DownloadableMetadata {
        id,
        version: Some(version),
        download_type: DownloadType::Game,
    };

    match process_manager_lock.launch_process(meta) {
        Ok(_) => {}
        Err(e) => return UserValue::Err(e),
    };

    drop(process_manager_lock);
    drop(state_lock);

    UserValue::Ok(())
}

#[tauri::command]
pub fn kill_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> UserValue<(), ProcessError> {
    let meta = get_current_meta(&game_id).ok_or(ProcessError::NotInstalled)?;
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    process_manager_lock
        .kill_game(meta)
        .map_err(ProcessError::IOError)
        .into()
}
