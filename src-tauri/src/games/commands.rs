use std::sync::Mutex;

use tauri::AppHandle;

use crate::{
    AppState,
    database::{
        db::borrow_db_checked,
        models::data::GameVersion,
    },
    error::{library_error::LibraryError, remote_access_error::RemoteAccessError},
    games::library::{
        fetch_game_logic_offline, fetch_library_logic_offline, get_current_meta,
        uninstall_game_logic,
    },
    offline,
};

use super::{
    library::{
        FetchGameStruct, Game, fetch_game_logic, fetch_game_verion_options_logic,
        fetch_library_logic,
    },
    state::{GameStatusManager, GameStatusWithTransient},
};

#[tauri::command]
pub fn fetch_library(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    offline!(
        state,
        fetch_library_logic,
        fetch_library_logic_offline,
        state
    )
}

#[tauri::command]
pub fn fetch_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    offline!(
        state,
        fetch_game_logic,
        fetch_game_logic_offline,
        game_id,
        state
    )
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
pub fn fetch_game_verion_options(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    fetch_game_verion_options_logic(game_id, state)
}
