
use log::info;
use tauri::AppHandle;


#[tauri::command]
pub fn quit(app: tauri::AppHandle) {
    cleanup_and_exit(&app);
}

pub fn cleanup_and_exit(app: &AppHandle) {
    info!("exiting drop application...");

    app.exit(0);
}
