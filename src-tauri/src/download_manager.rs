use database::DownloadableMetadata;
use download_manager::DOWNLOAD_MANAGER;

#[tauri::command]
pub async fn pause_downloads() {
    DOWNLOAD_MANAGER.pause_downloads().await;
}

#[tauri::command]
pub async fn resume_downloads() {
    DOWNLOAD_MANAGER.resume_downloads().await;
}

#[tauri::command]
pub async fn move_download_in_queue(old_index: usize, new_index: usize) {
    DOWNLOAD_MANAGER.rearrange(old_index, new_index).await;
}

#[tauri::command]
pub async fn cancel_game(meta: DownloadableMetadata) {
    DOWNLOAD_MANAGER.cancel(meta).await;
}
