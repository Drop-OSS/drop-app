use crate::{lock, AppState};

#[tauri::command]
pub fn fetch_state(
    state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>,
) -> Result<String, String> {
    let guard = lock!(state);
    let cloned_state = serde_json::to_string(&guard.clone()).map_err(|e| e.to_string())?;
    drop(guard);
    Ok(cloned_state)
}

#[tauri::command]
pub fn quit(app: tauri::AppHandle, state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>) {
    cleanup_and_exit(&app, &state);
}

pub fn cleanup_and_exit(app: &AppHandle, state: &tauri::State<'_, std::sync::Mutex<AppState<'_>>>) {
    debug!("cleaning up and exiting application");
    let download_manager = lock!(state).download_manager.clone();
    match download_manager.ensure_terminated() {
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
    toggle_autostart_logic(app, enabled)
}

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, tauri_plugin_autostart::Error> {
    get_autostart_enabled_logic(app)
}
