use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    database::{db::borrow_db_checked, models::data::GameDownloadStatus},
    download_manager::{
        download_manager_frontend::DownloadManagerSignal, downloadable::Downloadable,
    },
    error::download_manager_error::DownloadManagerError,
    AppState,
};

use super::download_agent::GameDownloadAgent;

#[tauri::command]
pub async fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), DownloadManagerError<DownloadManagerSignal>> {
    let sender = state.lock().unwrap().download_manager.get_sender();
    let game_download_agent = Arc::new(Box::new(GameDownloadAgent::new_from_index(
        game_id,
        game_version,
        install_dir,
        sender,
    ).await) as Box<dyn Downloadable + Send + Sync>);
    Ok(state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)?)
}

#[tauri::command]
pub async fn resume_download(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), DownloadManagerError<DownloadManagerSignal>> {
    let s = borrow_db_checked().await
        .applications
        .game_statuses
        .get(&game_id)
        .unwrap()
        .clone();

    let (version_name, install_dir) = match s {
        GameDownloadStatus::Remote {} => unreachable!(),
        GameDownloadStatus::SetupRequired { .. } => unreachable!(),
        GameDownloadStatus::Installed { .. } => unreachable!(),
        GameDownloadStatus::PartiallyInstalled {
            version_name,
            install_dir,
        } => (version_name, install_dir),
    };
    let sender = state.lock().unwrap().download_manager.get_sender();
    let parent_dir: PathBuf = install_dir.into();
    let game_download_agent = Arc::new(Box::new(GameDownloadAgent::new(
        game_id,
        version_name.clone(),
        parent_dir.parent().unwrap().to_path_buf(),
        sender,
    )) as Box<dyn Downloadable + Send + Sync>);
    Ok(state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)?)
}
