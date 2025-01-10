use std::sync::Mutex;

use tauri::AppHandle;

use crate::{
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
pub fn fetch_library(app: AppHandle) -> Result<Vec<Game>, String> {
    fetch_library_logic(app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fetch_game(game_id: String, app: tauri::AppHandle) -> Result<FetchGameStruct, String> {
    let result = fetch_game_logic(game_id, app);

    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }

    Ok(result.unwrap())
}

#[tauri::command]
pub fn fetch_game_status(id: String) -> Result<GameStatusWithTransient, String> {
    let status = GameStatusManager::fetch_state(&id);

    Ok(status)
}

#[tauri::command]
pub fn uninstall_game(game_id: String, app_handle: AppHandle) -> Result<(), String> {
    let meta = get_current_meta(&game_id)?;
    println!("{:?}", meta);
    uninstall_game_logic(meta, &app_handle);

    Ok(())
}

#[tauri::command]
pub fn fetch_game_verion_options(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersionOption>, String> {
    fetch_game_verion_options_logic(game_id, state).map_err(|e| e.to_string())
}
