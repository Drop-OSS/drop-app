use log::info;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn toggle_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
        info!("Enabled autostart");
    } else {
        manager.disable().map_err(|e| e.to_string())?;
        info!("Disabled autostart");
    }
    Ok(())
}

#[tauri::command]
pub async fn get_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    let manager = app.autolaunch();
    manager.is_enabled().map_err(|e| e.to_string())
} 