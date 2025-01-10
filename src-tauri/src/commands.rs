use tauri::AppHandle;

use crate::{
    autostart::{get_autostart_enabled_logic, toggle_autostart_logic},
    AppState,
};

#[tauri::command]
pub fn fetch_state(
    state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>,
) -> Result<String, String> {
    let guard = state.lock().unwrap();
    let cloned_state = serde_json::to_string(&guard.clone()).map_err(|e| e.to_string())?;
    drop(guard);
    Ok(cloned_state)
}
#[tauri::command]
pub fn toggle_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    toggle_autostart_logic(app, enabled)
}

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, tauri_plugin_autostart::Error> {
    get_autostart_enabled_logic(app)
}
