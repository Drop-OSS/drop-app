use std::{path::PathBuf, sync::Arc};

use database::{GameDownloadStatus, borrow_db_checked};
use download_manager::{
    DOWNLOAD_MANAGER, downloadable::Downloadable, error::ApplicationDownloadError,
};
use games::downloads::download_agent::GameDownloadAgent;

#[tauri::command]
pub async fn download_game(
    game_id: String,
    game_version: String,
    install_dir: usize,
) -> Result<(), ApplicationDownloadError> {
    let sender = { DOWNLOAD_MANAGER.get_sender().clone() };

    let game_download_agent = GameDownloadAgent::new_from_index(
        game_id.clone(),
        game_version.clone(),
        install_dir,
        sender,
    )
    .await?;

    let game_download_agent =
        Arc::new(Box::new(game_download_agent) as Box<dyn Downloadable + Send + Sync>);

    DOWNLOAD_MANAGER
        .queue_download(game_download_agent.clone())
        .unwrap();

    Ok(())
}

#[tauri::command]
pub async fn resume_download(game_id: String) -> Result<(), ApplicationDownloadError> {
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

    let sender = DOWNLOAD_MANAGER.get_sender();
    let parent_dir: PathBuf = install_dir.into();

    let game_download_agent = Arc::new(Box::new(
        GameDownloadAgent::new(
            game_id,
            version_name.clone(),
            parent_dir
                .parent()
                .unwrap_or_else(|| {
                    panic!("Failed to get parent directry of {}", parent_dir.display())
                })
                .to_path_buf(),
            sender,
        )
        .await?,
    ) as Box<dyn Downloadable + Send + Sync>);

    DOWNLOAD_MANAGER
        .queue_download(game_download_agent)
        .unwrap();
    Ok(())
}
