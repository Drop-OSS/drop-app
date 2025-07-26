use crate::{database::models::data::DownloadableMetadata, DropFunctionState};

#[tauri::command]
pub async fn pause_downloads(state: tauri::State<'_, DropFunctionState<'_>>) -> Result<(), ()> {
    state.lock().await.download_manager.pause_downloads();
    Ok(())
}

#[tauri::command]
pub async fn resume_downloads(state: tauri::State<'_, DropFunctionState<'_>>) -> Result<(), ()> {
    state.lock().await.download_manager.resume_downloads();
    Ok(())
}

#[tauri::command]
pub async fn move_download_in_queue(
    state: tauri::State<'_, DropFunctionState<'_>>,
    old_index: usize,
    new_index: usize,
) -> Result<(), ()> {
    state
        .lock()
        .await
        .download_manager
        .rearrange(old_index, new_index);
    Ok(())
}

#[tauri::command]
pub async fn cancel_game(state: tauri::State<'_, DropFunctionState<'_>>, meta: DownloadableMetadata) -> Result<(), ()> {
    state.lock().await.download_manager.cancel(meta);
    Ok(())
}
