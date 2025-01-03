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

#[tauri::command]
pub fn fetch_umu_info() -> Result<serde_json::Value, String> {
    let data_dir = DATA_ROOT_DIR.lock().unwrap().to_string_lossy().to_string();
    
    #[cfg(target_os = "linux")]
    let compat_info = {
        let data = DB.borrow_data().map_err(|e| e.to_string())?;
        let compat = data.compatibility.clone();
        json!({
            "enabled": compat.enabled,
            "runner": compat.runner,
            "prefix": compat.prefix_path
        })
    };

    #[cfg(not(target_os = "linux"))]
    let compat_info = json!(null);

    Ok(json!({
        "dataDir": data_dir,
        "compatibility": compat_info
    }))
} 
