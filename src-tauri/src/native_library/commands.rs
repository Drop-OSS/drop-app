use std::sync::Mutex;

use drop_database::{borrow_db_checked, models::data::GameVersion, runtime_models::Game};
use drop_errors::{library_error::LibraryError, remote_access_error::RemoteAccessError};
use drop_native_library::{library::{fetch_game_logic, fetch_game_logic_offline, fetch_game_version_options_logic, fetch_library_logic, fetch_library_logic_offline, get_current_meta, uninstall_game_logic, FetchGameStruct}, state::{GameStatusManager, GameStatusWithTransient}};
use tauri::AppHandle;

use crate::{AppState, offline};

#[tauri::command]
pub async fn fetch_library(
    state: tauri::State<'_, Mutex<AppState>>,
    hard_refresh: Option<bool>,
) -> Result<Vec<Game>, RemoteAccessError> {
    offline!(
        state,
        fetch_library_logic,
        fetch_library_logic_offline,
        hard_refresh
    )
    .await
}

#[tauri::command]
pub async fn fetch_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    offline!(
        state,
        fetch_game_logic,
        fetch_game_logic_offline,
        game_id
    )
    .await
}

#[tauri::command]
pub fn fetch_game_status(id: String) -> GameStatusWithTransient {
    let db_handle = borrow_db_checked();
    GameStatusManager::fetch_state(&id, &db_handle)
}

#[tauri::command]
pub fn uninstall_game(game_id: String, app_handle: AppHandle) -> Result<(), LibraryError> {
    let meta = match get_current_meta(&game_id) {
        Some(data) => data,
        None => return Err(LibraryError::MetaNotFound(game_id)),
    };
    uninstall_game_logic(meta, &app_handle);

    Ok(())
}

#[tauri::command]
pub async fn fetch_game_version_options(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    let all_versions = fetch_game_version_options_logic(game_id).await?;

    let state_lock = state.lock().unwrap();
    let process_manager_lock = state_lock.process_manager.lock().unwrap();
    let data: Vec<GameVersion> = all_versions
        .into_iter()
        .filter(|v| {
            process_manager_lock
                .valid_platform(&v.platform)
                .unwrap()
        })
        .collect();
    drop(process_manager_lock);
    drop(state_lock);

    Ok(data)
}
