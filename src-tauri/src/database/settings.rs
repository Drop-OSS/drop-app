use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub autostart: bool,
    pub max_download_threads: usize,
    pub force_offline: bool
    // ... other settings ...
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            max_download_threads: 4,
            force_offline: false
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
