use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tauri::AppHandle;

use crate::{
    AppState,
    database::{
        db::{borrow_db_checked, borrow_db_mut_checked},
        models::data::{GameDownloadStatus, v1::ApplicationTransientStatus},
    },
    download_manager::downloadable::Downloadable,
    error::application_download_error::ApplicationDownloadError,
    games::{library::push_game_update, state::GameStatusManager},
};

use super::download_agent::GameDownloadAgent;

#[tauri::command]
pub async fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
    app_handle: AppHandle,
) -> Result<(), ApplicationDownloadError> {
    let sender = { state.lock().unwrap().download_manager.get_sender().clone() };

    let game_download_agent =
        GameDownloadAgent::new_from_index(game_id.clone(), game_version.clone(), install_dir, sender).await?;

    let game_download_agent =
        Arc::new(Box::new(game_download_agent) as Box<dyn Downloadable + Send + Sync>);
    state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent.clone())
        .unwrap();

    Ok(())
}

#[tauri::command]
pub async fn resume_download(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), ApplicationDownloadError> {
    let s = borrow_db_checked()
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

    let game_download_agent = Arc::new(Box::new(
        GameDownloadAgent::new(
            game_id,
            version_name.clone(),
            parent_dir.parent().unwrap().to_path_buf(),
            sender,
        )
        .await?,
    ) as Box<dyn Downloadable + Send + Sync>);

    state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)
        .unwrap();
    Ok(())
}
