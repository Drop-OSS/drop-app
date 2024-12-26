use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex},
};

use http::StatusCode;
use log::{info, warn};
use serde::Deserialize;
use url::{ParseError, Url};

use crate::{AppState, AppStatus, DB};

#[derive(Debug, Clone)]
pub enum RemoteAccessError {
    FetchError(Arc<reqwest::Error>),
    ParsingError(ParseError),
    InvalidCodeError(u16),
    InvalidEndpoint,
    HandshakeFailed,
    GameNotFound,
    InvalidResponse,
    InvalidRedirect,
    ManifestDownloadFailed(StatusCode, String),
    OutOfSync,
}

impl Display for RemoteAccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteAccessError::FetchError(error) => write!(
                f,
                "{}: {}",
                error,
                error
                    .source()
                    .map(|e| e.to_string())
                    .or_else(|| Some("Unknown error".to_string()))
                    .unwrap()
            ),
            RemoteAccessError::ParsingError(parse_error) => {
                write!(f, "{}", parse_error)
            }
            RemoteAccessError::InvalidCodeError(error) => write!(f, "Invalid HTTP code {}", error),
            RemoteAccessError::InvalidEndpoint => write!(f, "Invalid drop endpoint"),
            RemoteAccessError::HandshakeFailed => write!(f, "Failed to complete handshake"),
            RemoteAccessError::GameNotFound => write!(f, "Could not find game on server"),
            RemoteAccessError::InvalidResponse => write!(f, "Server returned an invalid response"),
            RemoteAccessError::InvalidRedirect => write!(f, "Server redirect was invalid"),
            RemoteAccessError::ManifestDownloadFailed(status, response) => write!(
                f,
                "Failed to download game manifest: {} {}",
                status, response
            ),
            RemoteAccessError::OutOfSync => write!(f, "Server's and client's time are out of sync. Please ensure they are within at least 30 seconds of each other."),
        }
    }
}

impl From<reqwest::Error> for RemoteAccessError {
    fn from(err: reqwest::Error) -> Self {
        RemoteAccessError::FetchError(Arc::new(err))
    }
}
impl From<ParseError> for RemoteAccessError {
    fn from(err: ParseError) -> Self {
        RemoteAccessError::ParsingError(err)
    }
}
impl From<u16> for RemoteAccessError {
    fn from(err: u16) -> Self {
        RemoteAccessError::InvalidCodeError(err)
    }
}

impl std::error::Error for RemoteAccessError {}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropServerError {
    pub status_code: usize,
    pub status_message: String,
    pub message: String,
    pub url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DropHealthcheck {
    app_name: String,
}

async fn use_remote_logic<'a>(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'a>>>,
) -> Result<(), RemoteAccessError> {
    info!("connecting to url {}", url);
    let base_url = Url::parse(&url)?;

    // Test Drop url
    let test_endpoint = base_url.join("/api/v1")?;
    let response = reqwest::get(test_endpoint.to_string()).await?;

    let result = response.json::<DropHealthcheck>().await?;

    if result.app_name != "Drop" {
        warn!("user entered drop endpoint that connected, but wasn't identified as Drop");
        return Err(RemoteAccessError::InvalidEndpoint);
    }

    let mut app_state = state.lock().unwrap();
    app_state.status = AppStatus::SignedOut;
    drop(app_state);

    let mut db_state = DB.borrow_data_mut().unwrap();
    db_state.base_url = base_url.to_string();
    drop(db_state);

    DB.save().unwrap();

    Ok(())
}

#[tauri::command]
pub async fn use_remote<'a>(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'a>>>,
) -> Result<(), String> {
    let result = use_remote_logic(url, state).await;

    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn gen_drop_url(path: String) -> Result<String, String> {
    let base_url = {
        let handle = DB.borrow_data().unwrap();

        if handle.base_url.is_empty() {
            return Ok("".to_string());
        };

        Url::parse(&handle.base_url).unwrap()
    };

    let url = base_url.join(&path).unwrap();

    Ok(url.to_string())
}
