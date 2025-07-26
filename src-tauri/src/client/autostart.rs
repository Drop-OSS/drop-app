use crate::database::db::{borrow_db_checked, borrow_db_mut_checked};
use log::debug;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub async fn toggle_autostart_logic(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
        debug!("enabled autostart");
    } else {
        manager.disable().map_err(|e| e.to_string())?;
        debug!("eisabled autostart");
    }

    // Store the state in DB
    let mut db_handle = borrow_db_mut_checked().await;
    db_handle.settings.autostart = enabled;
    drop(db_handle);

    Ok(())
}

pub async fn get_autostart_enabled_logic(
    app: AppHandle,
) -> Result<bool, tauri_plugin_autostart::Error> {
    // First check DB state
    let db_handle = borrow_db_checked().await;
    let db_state = db_handle.settings.autostart;
    drop(db_handle);

    // Get actual system state
    let manager = app.autolaunch();
    let system_state = manager.is_enabled()?;

    // If they don't match, sync to DB state
    if db_state != system_state {
        if db_state {
            manager.enable()?;
        } else {
            manager.disable()?;
        }
    }

    Ok(db_state)
}

// New function to sync state on startup
pub async fn sync_autostart_on_startup(app: &AppHandle) -> Result<(), String> {
    let db_handle = borrow_db_checked().await;
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
#[tauri::command]
pub async fn toggle_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    toggle_autostart_logic(app, enabled).await
}

#[tauri::command]
pub async fn get_autostart_enabled(app: AppHandle) -> Result<bool, tauri_plugin_autostart::Error> {
    get_autostart_enabled_logic(app).await
}
