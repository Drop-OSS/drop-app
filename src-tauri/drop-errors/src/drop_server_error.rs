use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerError {
    pub status_code: usize,
    pub status_message: String,
    // pub message: String,
    // pub url: String,
}
