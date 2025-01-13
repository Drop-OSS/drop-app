use std::sync::Mutex;

use tauri::AppHandle;

use crate::{
    error::{
        library_error::LibraryError, remote_access_error::RemoteAccessError, user_error::UserValue,
    },
    games::library::{get_current_meta, uninstall_game_logic},
    AppState,
};

use super::{
    library::{
        fetch_game_logic, fetch_game_verion_options_logic, fetch_library_logic, FetchGameStruct,
        Game, GameVersionOption,
    },
    state::{GameStatusManager, GameStatusWithTransient},
};

#[tauri::command]
pub fn fetch_library(app: AppHandle) -> UserValue<Vec<Game>, RemoteAccessError> {
    fetch_library_logic(app).into()
}

#[tauri::command]
pub fn fetch_game(
    game_id: String,
    app: tauri::AppHandle,
) -> UserValue<FetchGameStruct, RemoteAccessError> {
    fetch_game_logic(game_id, app).into()
}

#[tauri::command]
pub fn fetch_game_status(id: String) -> GameStatusWithTransient {
    GameStatusManager::fetch_state(&id)
}

#[tauri::command]
pub fn uninstall_game(game_id: String, app_handle: AppHandle) -> UserValue<(), LibraryError> {
    let meta = match get_current_meta(&game_id) {
        Some(data) => data,
        None => return UserValue::Err(LibraryError::MetaNotFound(game_id)),
    };
    println!("{:?}", meta);
    uninstall_game_logic(meta, &app_handle);

    UserValue::Ok(())
}

#[tauri::command]
pub fn fetch_game_verion_options(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> UserValue<Vec<GameVersionOption>, RemoteAccessError> {
    fetch_game_verion_options_logic(game_id, state).into()
}
