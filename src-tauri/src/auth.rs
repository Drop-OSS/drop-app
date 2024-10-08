use std::{
    borrow::{Borrow, BorrowMut},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use tauri::{App, AppHandle, Emitter, Error, EventLoopMessage, Wry};
use url::Url;

use crate::{AppStatus, User, DB};

#[derive(Serialize)]
struct InitiateRequestBody {
    name: String,
    platform: String,
}

pub async fn recieve_handshake(app: AppHandle, path: String) {
    // Tell the app we're connecting
    app.emit("auth/connecting", ()).unwrap();

    // TODO
}

#[tauri::command]
pub async fn auth_initiate<'a>() -> Result<(), String> {
    let base_url = {
        let db_lock = DB.borrow_data().unwrap();
        Url::parse(&db_lock.base_url.clone()).unwrap()
    };

    let current_os_info = os_info::get();

    let endpoint = base_url.join("/api/v1/client/initiate").unwrap();
    let body = InitiateRequestBody {
        name: format!("Drop Desktop Client"),
        platform: current_os_info.os_type().to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(endpoint.to_string())
        .json(&body)
        .send()
        .await
        .unwrap();

    let redir_url = response.text().await.unwrap();
    let complete_redir_url = base_url.join(&redir_url).unwrap();

    webbrowser::open(&complete_redir_url.to_string()).unwrap();

    return Ok(());
}

pub fn setup() -> Result<(AppStatus, Option<User>), Error> {
    let data = DB.borrow_data().unwrap();

    // If we have certs, exit for now
    if data.certs.is_some() {
        // TODO: check if it's still valid, and fetch user information
        return Ok((AppStatus::SignedInNeedsReauth, None));
    }

    drop(data);

    auth_initiate();

    return Ok((AppStatus::SignedOut, None));
}
