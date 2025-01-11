use crate::{DATA_ROOT_DIR, DB};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemData {
    client_id: String,
    base_url: String,
    data_dir: String,
    log_level: String,
}

#[tauri::command]
pub fn fetch_system_data() -> Result<SystemData, String> {
    let db_handle = DB.borrow_data().map_err(|e| e.to_string())?;
    let system_data = SystemData {
        client_id: db_handle.auth.as_ref().unwrap().client_id.clone(),
        base_url: db_handle.base_url.clone(),
        data_dir: DATA_ROOT_DIR.lock().unwrap().to_string_lossy().to_string(),
        log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    };
    drop(db_handle);

    Ok(system_data)
}
