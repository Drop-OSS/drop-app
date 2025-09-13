use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DropServerError {
    pub status_code: usize,
    pub status_message: String,
    // pub message: String,
    // pub url: String,
}
