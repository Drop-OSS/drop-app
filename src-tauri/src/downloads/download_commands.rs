use std::sync::{Arc, Mutex};

use log::info;
use rayon::spawn;

use crate::{downloads::download_agent::GameDownloadAgent, AppState};

#[tauri::command]
pub fn download_game(
    game_id: String,
    game_version: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    info!("beginning game download...");

    let mut download_agent = GameDownloadAgent::new(game_id.clone(), game_version.clone(), 0);
    // Setup download requires mutable
    download_agent.setup_download().unwrap();

    let mut lock: std::sync::MutexGuard<'_, AppState> = state.lock().unwrap();
    let download_agent_ref = Arc::new(download_agent);
    lock.game_downloads
        .insert(game_id, download_agent_ref.clone());

    // Run it in another thread
    spawn(move || {
        // Run doesn't require mutable
        download_agent_ref.clone().run();
    });

    Ok(())
}

#[tauri::command]
pub fn get_game_download_progress(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<f64, String> {
    let da = use_download_agent(state, game_id)?;

    let progress = &da.progress;
    let current: f64 = progress
        .current
        .fetch_add(0, std::sync::atomic::Ordering::Relaxed) as f64;
    let max = progress.max as f64;

    let current_progress = current / max;

    Ok(current_progress)
}

fn use_download_agent(
    state: tauri::State<'_, Mutex<AppState>>,
    game_id: String,
) -> Result<Arc<GameDownloadAgent>, String> {
    let lock = state.lock().unwrap();
    let download_agent = lock.game_downloads.get(&game_id).ok_or("Invalid game ID")?;
    Ok(download_agent.clone()) // Clones the Arc, not the underlying data structure
}
