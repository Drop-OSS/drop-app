use log::{debug, info};
use tauri::AppHandle;

use crate::AppState;

#[tauri::command]
pub fn quit(app: tauri::AppHandle, state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>) {
    cleanup_and_exit(&app, &state);
}

pub fn cleanup_and_exit(app: &AppHandle, state: &tauri::State<'_, std::sync::Mutex<AppState<'_>>>) {
    debug!("Cleaning up and exiting application");
    let download_manager = state.lock().unwrap().download_manager.clone();
    match download_manager.ensure_terminated() {
        Ok(_) => {},
        Err(e) => panic!("{:?}", e),
    }

    app.exit(0);
}
