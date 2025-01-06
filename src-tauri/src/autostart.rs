use crate::DB;
use log::debug;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn toggle_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
        debug!("Enabled autostart");
    } else {
        manager.disable().map_err(|e| e.to_string())?;
        debug!("Disabled autostart");
    }

    // Store the state in DB
    let mut db_handle = DB.borrow_data_mut().map_err(|e| e.to_string())?;
    db_handle.settings.autostart = enabled;
    drop(db_handle);
    DB.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    // First check DB state
    let db_handle = DB.borrow_data().map_err(|e| e.to_string())?;
    let db_state = db_handle.settings.autostart;
    drop(db_handle);

    // Get actual system state
    let manager = app.autolaunch();
    let system_state = manager.is_enabled().map_err(|e| e.to_string())?;

    // If they don't match, sync to DB state
    if db_state != system_state {
        if db_state {
            manager.enable().map_err(|e| e.to_string())?;
        } else {
            manager.disable().map_err(|e| e.to_string())?;
        }
    }

    Ok(db_state)
}

// New function to sync state on startup
pub fn sync_autostart_on_startup(app: &AppHandle) -> Result<(), String> {
    let db_handle = DB.borrow_data().map_err(|e| e.to_string())?;
    let should_be_enabled = db_handle.settings.autostart;
    drop(db_handle);

    let manager = app.autolaunch();
    let current_state = manager.is_enabled().map_err(|e| e.to_string())?;

    if current_state != should_be_enabled {
        if should_be_enabled {
            manager.enable().map_err(|e| e.to_string())?;
            debug!("Synced autostart: enabled");
        } else {
            manager.disable().map_err(|e| e.to_string())?;
            debug!("Synced autostart: disabled");
        }
    }

    Ok(())
}
