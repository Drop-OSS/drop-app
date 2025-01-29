use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::{
    database::db::GameVersion, error::{library_error::LibraryError, remote_access_error::RemoteAccessError}, games::library::{get_current_meta, uninstall_game_logic}, offline, AppState
};

use super::{
    library::{
        fetch_game_logic, fetch_game_verion_options_logic, fetch_library_logic, FetchGameStruct,
        Game,
    },
    state::{GameStatusManager, GameStatusWithTransient},
};

#[tauri::command]
pub fn fetch_library(app: AppHandle) -> Result<Vec<Game>, RemoteAccessError> {
    let state = app.state::<Mutex<AppState>>();
    offline!(state, fetch_library_logic, fetch_library_logic, app)
}

#[tauri::command]
pub fn fetch_game(
    game_id: String,
    app: tauri::AppHandle,
) -> Result<FetchGameStruct, RemoteAccessError> {
    fetch_game_logic(game_id, app)
}

#[tauri::command]
pub fn fetch_game_status(id: String) -> GameStatusWithTransient {
    GameStatusManager::fetch_state(&id)
}

#[tauri::command]
pub fn uninstall_game(game_id: String, app_handle: AppHandle) -> Result<(), LibraryError> {
    let meta = match get_current_meta(&game_id) {
        Some(data) => data,
        None => return Err(LibraryError::MetaNotFound(game_id)),
    };
    println!("{:?}", meta);
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
