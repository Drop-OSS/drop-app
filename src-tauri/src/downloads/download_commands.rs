use std::{
    sync::{atomic::Ordering, Arc, Mutex},
    thread,
};

use log::info;

use crate::{downloads::download_agent::GameDownloadAgent, AppState};

use super::download_agent::{GameDownloadError, GameDownloadState};

#[tauri::command]
pub async fn queue_game_download(
    game_id: String,
    game_version: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), GameDownloadError> {
    info!("Queuing Game Download");
    let download_agent = Arc::new(GameDownloadAgent::new(
        game_id.clone(),
        game_version.clone(),
    ));
    download_agent.queue().await?;

    let mut queue = state.lock().unwrap();
    queue.game_downloads.insert(game_id, download_agent);
    Ok(())
}

#[tauri::command]
pub async fn start_game_downloads(
    max_threads: usize,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), GameDownloadError> {
    info!("Downloading Games");
    let lock = state.lock().unwrap();
    let mut game_downloads = lock.game_downloads.clone();
    drop(lock);
    thread::spawn(move || loop {
        let mut current_id = String::new();
        let mut download_agent = None;
        {
            for (id, agent) in &game_downloads {
                if agent.get_state() == GameDownloadState::Queued {
                    download_agent = Some(agent.clone());
                    current_id = id.clone();
                    info!("Got queued game to download");
                    break;
                }
            }
            if download_agent.is_none() {
                info!("No more games left to download");
                return;
            }
        };
        info!("Downloading game");
        {
            start_game_download(max_threads, download_agent.unwrap()).unwrap();
            game_downloads.remove_entry(&current_id);
        }
    });
    info!("Spawned download");
    Ok(())
}

pub fn start_game_download(
    max_threads: usize,
    download_agent: Arc<GameDownloadAgent>,
) -> Result<(), GameDownloadError> {
    info!("Triggered Game Download");

    download_agent.ensure_manifest_exists()?;

    let local_manifest = {
        let manifest = download_agent.manifest.lock().unwrap();
        (*manifest).clone().unwrap()
    };

    download_agent
        .generate_job_contexts(
            &local_manifest,
            download_agent.version.clone(),
            download_agent.id.clone(),
        )
        .unwrap();

    download_agent.begin_download(max_threads).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn cancel_specific_game_download(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<(), String> {
    info!("called stop_specific_game_download");
    let status = get_game_download(state, game_id).change_state(GameDownloadState::Cancelled);

    //TODO: Drop the game download instance

    info!("Stopping callback");

    Ok(())
}

#[tauri::command]
pub async fn get_game_download_progress(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<f64, String> {
    let progress = get_game_download(state, game_id).progress.get_progress_percentage();
    info!("{}", progress);
    Ok(progress)
}

#[tauri::command]
pub async fn pause_game_download(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<(), String> {
    get_game_download(state, game_id).change_state(GameDownloadState::Paused);
    
    Ok(())
}

#[tauri::command]
pub async fn resume_game_download(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<(), String> {
    get_game_download(state, game_id).change_state(GameDownloadState::Downloading);
    
    Ok(())
}

fn get_game_download(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Arc<GameDownloadAgent> {
    let lock = state.lock().unwrap();
    let download_agent = lock.game_downloads.get(&game_id).unwrap();
    download_agent.clone()
}