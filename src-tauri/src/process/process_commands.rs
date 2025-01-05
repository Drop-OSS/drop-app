use std::sync::Mutex;

use crate::{
    db::GameDownloadStatus,
    download_manager::downloadable_metadata::{DownloadType, DownloadableMetadata},
    games::library::get_current_meta,
    AppState, DB,
};

#[tauri::command]
pub fn launch_game(id: String, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
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
        Some(GameDownloadStatus::Installed {
            version_name,
            install_dir,
        }) => version_name,
        Some(GameDownloadStatus::SetupRequired {
            version_name,
            install_dir,
        }) => return Err(String::from("Game setup still required")),
        _ => return Err(String::from("Game not installed")),
    };

    let meta = DownloadableMetadata {
        id,
        version: Some(version),
        download_type: DownloadType::Game,
    };

    process_manager_lock.launch_process(meta)?;

    drop(process_manager_lock);
    drop(state_lock);

    Ok(())
}

#[tauri::command]
pub fn kill_game(game_id: String, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let meta = get_current_meta(&game_id)?;
    let state_lock = state.lock().unwrap();
    let mut process_manager_lock = state_lock.process_manager.lock().unwrap();
    process_manager_lock
        .kill_game(meta)
        .map_err(|x| x.to_string())
}
