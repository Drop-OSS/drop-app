use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex},
};

use http::StatusCode;
use log::{info, warn};
use serde::Deserialize;
use url::{ParseError, Url};

use crate::{error::remote_access_error::RemoteAccessError, AppState, AppStatus, DB};

#[derive(Deserialize, Debug, Clone)]
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

fn use_remote_logic(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), RemoteAccessError> {
    info!("connecting to url {}", url);
    let base_url = Url::parse(&url)?;

    // Test Drop url
    let test_endpoint = base_url.join("/api/v1")?;
    let response = reqwest::blocking::get(test_endpoint.to_string())?;

    let result: DropHealthcheck = response.json()?;

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
pub fn use_remote<'a>(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'a>>>,
) -> Result<(), String> {
    let result = use_remote_logic(url, state);

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
