use database::DownloadableMetadata;
use download_manager::DOWNLOAD_MANAGER;

#[tauri::command]
pub fn pause_downloads() {
    DOWNLOAD_MANAGER.pause_downloads();
}

#[tauri::command]
pub fn resume_downloads() {
    DOWNLOAD_MANAGER.resume_downloads();
}

#[tauri::command]
pub fn move_download_in_queue(old_index: usize, new_index: usize) {
    DOWNLOAD_MANAGER.rearrange(old_index, new_index);
}

#[tauri::command]
pub fn cancel_game(meta: DownloadableMetadata) {
    DOWNLOAD_MANAGER.cancel(meta);
}
