use std::{path::PathBuf, sync::Arc};

use database::{
    DownloadType, DownloadableMetadata, GameDownloadStatus, borrow_db_checked,
    models::data::{InstalledGameType, UserConfiguration},
    platform::Platform,
};
use download_manager::{
    DOWNLOAD_MANAGER, downloadable::Downloadable, error::ApplicationDownloadError,
};
use games::downloads::download_agent::GameDownloadAgent;

#[tauri::command]
pub async fn download_game(
    game_id: String,
    version_id: String,
    target_platform: Platform,
    install_dir: usize,
    enable_updates: bool,
) -> Result<(), ApplicationDownloadError> {
    let sender = { DOWNLOAD_MANAGER.get_sender().clone() };

    let meta = DownloadableMetadata {
        id: game_id,
        version: version_id,
        target_platform,
        download_type: DownloadType::Game,
    };

    {
        let db = borrow_db_checked();
        let status = db.applications.transient_statuses.get(&meta);

        if status.is_some() {
            return Ok(());
        }
    };

    let configuration = UserConfiguration {
        enable_updates,
        ..Default::default()
    };

    let base_dir = {
        let db_lock = borrow_db_checked();

        db_lock.applications.install_dirs[install_dir].clone()
    };

    let game_download_agent = GameDownloadAgent::new(
        meta,
        base_dir,
        sender,
        DOWNLOAD_MANAGER.clone_depot_manager(),
        configuration,
    )
    .await?;

    let game_download_agent =
        Arc::new(Box::new(game_download_agent) as Box<dyn Downloadable + Send + Sync>);

    DOWNLOAD_MANAGER
        .queue_download(game_download_agent.clone())
        .await
        .unwrap();

    Ok(())
}

#[tauri::command]
pub async fn resume_download(game_id: String) -> Result<(), ApplicationDownloadError> {
    let (meta, (install_dir, configuration)) = {
        let db_lock = borrow_db_checked();
        let status = db_lock
            .applications
            .game_statuses
            .get(&game_id)
            .ok_or(ApplicationDownloadError::InvalidCommand)?
            .clone();

        let meta = db_lock
            .applications
            .installed_game_version
            .get(&game_id)
            .ok_or(ApplicationDownloadError::InvalidCommand)?
            .clone();

        let install_dir = match status {
            GameDownloadStatus::Installed {
                install_type: InstalledGameType::PartiallyInstalled { configuration },
                install_dir,
                ..
            } => Ok((install_dir, configuration)),
            _ => Err(ApplicationDownloadError::InvalidCommand),
        }?;
        (meta, install_dir)
    };

    let sender = DOWNLOAD_MANAGER.get_sender();

    let install_dir = PathBuf::from(install_dir);
    let install_dir = install_dir
        .parent()
        .expect("game somehow installed at root");

    let game_download_agent = Arc::new(Box::new(
        GameDownloadAgent::new(
            meta,
            install_dir.to_path_buf(),
            sender,
            DOWNLOAD_MANAGER.clone_depot_manager(),
            configuration,
        )
        .await?,
    ) as Box<dyn Downloadable + Send + Sync>);

    DOWNLOAD_MANAGER
        .queue_download(game_download_agent)
        .await
        .unwrap();
    Ok(())
}
