use crate::AppState;
use tauri::Manager;

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
pub async fn enter_fullscreen(app_handle: tauri::AppHandle) -> Result<(), String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    window.set_fullscreen(true).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn exit_fullscreen(app_handle: tauri::AppHandle) -> Result<(), String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn is_fullscreen(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    let is_fullscreen = window.is_fullscreen().map_err(|e| e.to_string())?;
    Ok(is_fullscreen)
}
