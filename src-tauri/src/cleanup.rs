use std::sync::Mutex;

use log::info;
use tauri::AppHandle;

use crate::AppState;

#[tauri::command]
pub fn quit(app: tauri::AppHandle) {
    cleanup_and_exit(&app);
}

pub fn cleanup_and_exit(app: &AppHandle) {
    info!("exiting drop application...");

    app.exit(0);
}
