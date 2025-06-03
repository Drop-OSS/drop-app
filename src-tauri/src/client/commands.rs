use std::sync::Arc;

use crate::{
    client::url_downloader::URLDownloader, download_manager::{download_manager::DownloadManagerSignal, downloadable::Downloadable}, error::download_manager_error::DownloadManagerError, AppState
};

#[tauri::command]
pub fn fetch_state(
    state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>,
) -> Result<String, String> {
    let guard = state.lock().unwrap();
    let cloned_state = serde_json::to_string(&guard.clone()).map_err(|e| e.to_string())?;
    drop(guard);
    Ok(cloned_state)
}

#[tauri::command]
pub fn queue_url_download(
    state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>,
    url: String
) -> Result<(), DownloadManagerError<DownloadManagerSignal>> {
    let sender = state.lock().unwrap().download_manager.get_sender();
    let game_download_agent = Arc::new(Box::new(URLDownloader::new(
        String::from("Test URL Download"),
        "/home/quexeky/Downloads/test_url_download",
        sender,
        url,
    )) as Box<dyn Downloadable + Send + Sync>);
    Ok(state
        .lock()
        .unwrap()
        .download_manager
        .queue_download(game_download_agent)?)
}
