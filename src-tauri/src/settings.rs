use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::DB;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub autostart: bool,
    pub max_download_threads: usize,
    // ... other settings ...
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            max_download_threads: 4
        }
    }
}
fn deserialize_into<T>(v: serde_json::Value, t: &mut T) -> Result<(), serde_json::Error>
    where T: for<'a> Deserialize<'a>
{
    *t = serde_json::from_value(v)?;
    Ok(())
}

#[tauri::command]
pub fn amend_settings(new_settings: Value) {
    let db_lock = DB.borrow_data_mut().unwrap();
    let mut current_settings = db_lock.settings.clone();
    let e = deserialize_into(new_settings, &mut current_settings);

    println!("Amend status: {:?}", e);
    println!("New settings: {:?}", current_settings);
}