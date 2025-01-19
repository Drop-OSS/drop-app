use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex},
};

use http::StatusCode;
use log::{debug, info, warn};
use serde::Deserialize;
use url::{ParseError, Url};

use crate::{database::db::{borrow_db_mut_checked, save_db}, error::remote_access_error::RemoteAccessError, AppState, AppStatus, DB};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DropHealthcheck {
    app_name: String,
}

pub fn use_remote_logic(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), RemoteAccessError> {
    debug!("connecting to url {}", url);
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

    let mut db_state = borrow_db_mut_checked();
    db_state.base_url = base_url.to_string();
    drop(db_state);

    save_db();

    Ok(())
}
