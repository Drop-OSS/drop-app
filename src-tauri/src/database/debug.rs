use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemData {
    client_id: String,
    base_url: String,
    data_dir: String,
    log_level: String,
}

impl SystemData {
    pub fn new(client_id: String, base_url: String, data_dir: String, log_level: String) -> Self {
        Self {
            client_id,
            base_url,
            data_dir,
            log_level,
        }
    }
}
