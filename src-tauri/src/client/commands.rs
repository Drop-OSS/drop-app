use crate::DropFunctionState;

#[tauri::command]
pub async fn fetch_state(
    state: tauri::State<'_, DropFunctionState<'_>>,
) -> Result<String, String> {
    let guard = state.lock().await;
    let cloned_state = serde_json::to_string(&guard.clone()).map_err(|e| e.to_string())?;
    drop(guard);
    Ok(cloned_state)
}
