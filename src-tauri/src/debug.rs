use crate::{DB, DATA_ROOT_DIR};
use crate::db::DatabaseImpls;
use serde_json::json;

#[tauri::command]
pub fn fetch_client_id() -> Result<Option<String>, String> {
    let data = DB.borrow_data().map_err(|e| e.to_string())?;
    Ok(data.auth.as_ref().map(|auth| auth.client_id.clone()))
}

#[tauri::command]
pub fn fetch_base_url() -> Result<Option<String>, String> {
    Ok(Some(DB.fetch_base_url().to_string()))
}

