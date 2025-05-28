use crate::AppState;

#[tauri::command]
pub fn fetch_state(
    state: tauri::State<'_, std::sync::Mutex<AppState<'_>>>,
) -> Result<String, String> {
    let guard = state.lock().unwrap();
    let cloned_state = serde_json::to_string(&guard.clone()).map_err(|e| e.to_string())?;
    drop(guard);
    Ok(cloned_state)
}
