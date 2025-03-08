use log::{info, warn};
use tauri::AppHandle;
use crate::update_checker;

#[tauri::command]
pub fn check_for_updates(app_handle: AppHandle) -> Result<(), String> {
    info!("Check for updates command received");
    let result = update_checker::check_for_updates(&app_handle)
        .map_err(|e| {
            warn!("Update check failed: {}", e);
            e.to_string()
        });
    info!("Update check completed");
    result
}
