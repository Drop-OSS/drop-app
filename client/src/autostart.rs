use database::borrow_db_checked;
use log::debug;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

// New function to sync state on startup
pub fn sync_autostart_on_startup(app: &AppHandle) -> Result<(), String> {
    let db_handle = borrow_db_checked();
    let should_be_enabled = db_handle.settings.autostart;
    drop(db_handle);

    let manager = app.autolaunch();
    let current_state = manager.is_enabled().map_err(|e| e.to_string())?;

    if current_state != should_be_enabled {
        if should_be_enabled {
            manager.enable().map_err(|e| e.to_string())?;
            debug!("synced autostart: enabled");
        } else {
            manager.disable().map_err(|e| e.to_string())?;
            debug!("synced autostart: disabled");
        }
    }

    Ok(())
}
