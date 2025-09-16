use std::sync::Mutex;

use drop_database::borrow_db_mut_checked;
use drop_errors::{library_error::LibraryError, process_error::ProcessError};
use serde::Deserialize;

use crate::AppState;

#[tauri::command]
pub fn launch_game(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();

    //let meta = DownloadableMetadata {
    //    id,
    //    version: Some(version),
    //    download_type: DownloadType::Game,
    //};

    match process_manager_lock.launch_process(id, state_lock.process_manager) {
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
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    process_manager_lock
        .kill_game(game_id)
        .map_err(ProcessError::IOError)
}

#[tauri::command]
pub fn open_process_logs(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ProcessError> {
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    process_manager_lock.open_process_logs(game_id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendGameOptions {
    pub launch_string: String,
}

#[tauri::command]
pub fn update_game_configuration(
    game_id: String,
    options: FrontendGameOptions,
) -> Result<(), LibraryError> {
    let mut handle = borrow_db_mut_checked();
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
