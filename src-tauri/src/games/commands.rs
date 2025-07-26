use serde::Deserialize;
use tauri::AppHandle;

use crate::{
    database::{db::borrow_db_mut_checked, models::data::GameVersion}, error::{library_error::LibraryError, remote_access_error::RemoteAccessError}, games::library::{
        fetch_game_logic_offline, fetch_library_logic_offline, get_current_meta,
        uninstall_game_logic,
    }, offline, DropFunctionState
};

use super::{
    library::{
        FetchGameStruct, Game, fetch_game_logic, fetch_game_verion_options_logic,
        fetch_library_logic,
    },
    state::{GameStatusManager, GameStatusWithTransient},
};

#[tauri::command]
pub async fn fetch_library(
    state: tauri::State<'_, DropFunctionState<'_>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    offline!(
        state,
        fetch_library_logic,
        fetch_library_logic_offline,
        state
    )
    .await
}

#[tauri::command]
pub async fn fetch_game(
    game_id: String,
    state: tauri::State<'_, DropFunctionState<'_>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    offline!(
        state,
        fetch_game_logic,
        fetch_game_logic_offline,
        game_id,
        state
    )
    .await
}

#[tauri::command]
pub async fn fetch_game_status(id: String) -> GameStatusWithTransient {
    GameStatusManager::fetch_state(&id).await
}

#[tauri::command]
pub async fn uninstall_game(game_id: String, app_handle: AppHandle) -> Result<(), LibraryError> {
    let meta = match get_current_meta(&game_id).await {
        Some(data) => data,
        None => return Err(LibraryError::MetaNotFound(game_id)),
    };
    uninstall_game_logic(meta, &app_handle).await;

    Ok(())
}

#[tauri::command]
pub async fn fetch_game_verion_options(
    game_id: String,
    state: tauri::State<'_, DropFunctionState<'_>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    fetch_game_verion_options_logic(game_id, state).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendGameOptions {
    launch_string: String,
}

#[tauri::command]
pub async fn update_game_configuration(
    game_id: String,
    options: FrontendGameOptions,
) -> Result<(), LibraryError> {
    let mut handle = borrow_db_mut_checked().await;
    let installed_version = handle
        .applications
        .installed_game_version
        .get(&game_id)
        .ok_or(LibraryError::MetaNotFound(game_id))?;

    let id = installed_version.id.clone();
    let version = installed_version.version.clone().unwrap();

    let mut existing_configuration = handle
        .applications
        .game_versions
        .get(&id)
        .unwrap()
        .get(&version)
        .unwrap()
        .clone();

    // Add more options in here
    existing_configuration.launch_command_template = options.launch_string;

    // Add no more options past here

    handle
        .applications
        .game_versions
        .get_mut(&id)
        .unwrap()
        .insert(version.to_string(), existing_configuration);

    Ok(())
}
