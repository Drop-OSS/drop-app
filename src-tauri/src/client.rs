use std::sync::nonpoison::Mutex;

use database::{borrow_db_checked, borrow_db_mut_checked};
use download_manager::DOWNLOAD_MANAGER;
use log::{debug, error};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::AppState;

#[tauri::command]
pub fn fetch_state(state: tauri::State<'_, Mutex<AppState>>) -> Result<String, String> {
    let guard = state.lock();
    let cloned_state = serde_json::to_string(&guard.clone()).map_err(|e| e.to_string())?;
    drop(guard);
    Ok(cloned_state)
}

#[tauri::command]
pub fn quit(app: tauri::AppHandle) {
    cleanup_and_exit(&app);
}

pub fn cleanup_and_exit(app: &AppHandle) {
    debug!("cleaning up and exiting application");
    match DOWNLOAD_MANAGER.ensure_terminated() {
        Ok(res) => match res {
            Ok(()) => debug!("download manager terminated correctly"),
            Err(()) => error!("download manager failed to terminate correctly"),
        },
        Err(e) => panic!("{e:?}"),
    }

    app.exit(0);
}

#[tauri::command]
pub fn toggle_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
        debug!("enabled autostart");
    } else {
        manager.disable().map_err(|e| e.to_string())?;
        debug!("eisabled autostart");
    }

    // Store the state in DB
    let mut db_handle = borrow_db_mut_checked();
    db_handle.settings.autostart = enabled;
    Ok(())
}

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, tauri_plugin_autostart::Error> {
    let db_handle = borrow_db_checked();
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
