use std::{
    borrow::{Borrow, BorrowMut},
    sync::Mutex,
};

use log::{info, warn};
use openssl::x509::store::HashDir;
use serde::Deserialize;
use tauri::async_runtime::handle;
use url::Url;

use crate::{AppState, AppStatus, DB};

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(e) => {
                return Err(format!(
                    "Invalid URL or Drop is inaccessible ({})",
                    e.to_string()
                ))
            }
        }
    };
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct DropHealthcheck {
    app_name: String,
}

#[tauri::command]
pub async fn use_remote<'a>(
    url: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    info!("connecting to url {}", url);
    let base_url = unwrap_or_return!(Url::parse(&url));

    // Test Drop url
    let test_endpoint = base_url.join("/api/v1").unwrap();
    let response = unwrap_or_return!(reqwest::get(test_endpoint.to_string()).await);

    let result = response.json::<DropHealthcheck>().await.unwrap();

    if result.app_name != "Drop" {
        warn!("user entered drop endpoint that connected, but wasn't identified as Drop");
        return Err("Not a valid Drop endpoint".to_string());
    }

    let mut app_state = state.lock().unwrap();
    app_state.status = AppStatus::SignedOut;
    drop(app_state);

    let mut db_state = DB.borrow_data_mut().unwrap();
    db_state.base_url = base_url.to_string();
    drop(db_state);

    DB.save().unwrap();

    return Ok(());
}

#[tauri::command]
pub fn gen_drop_url(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let base_url = {
        let handle = DB.borrow_data().unwrap();

        if handle.base_url.is_empty() {
            return Ok("".to_string());
        };

        Url::parse(&handle.base_url).unwrap()
    };

    let url = base_url.join(&path).unwrap();

    return Ok(url.to_string());
}
