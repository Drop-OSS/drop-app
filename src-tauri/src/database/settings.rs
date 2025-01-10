use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::DB;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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
// Ideally use pointers instead of a macro to assign the settings
// fn deserialize_into<T>(v: serde_json::Value, t: &mut T) -> Result<(), serde_json::Error>
//     where T: for<'a> Deserialize<'a>
// {
//     *t = serde_json::from_value(v)?;
//     Ok(())
// }


#[tauri::command]
pub fn amend_settings(new_settings: Value) {
    println!("{}", new_settings);
    let db_lock = DB.borrow_data_mut().unwrap();
    let mut current_settings = db_lock.settings.clone();
}