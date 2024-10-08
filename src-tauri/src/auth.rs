use std::{
    borrow::{Borrow, BorrowMut},
    sync::Mutex,
};

use log::info;
use serde::{Deserialize, Serialize};
use tauri::{App, AppHandle, Emitter, Error, EventLoopMessage, Manager, Wry};
use url::Url;

use crate::{data::DatabaseCerts, AppState, AppStatus, User, DB};

#[derive(Serialize)]
struct InitiateRequestBody {
    name: String,
    platform: String,
}

#[derive(Serialize)]
struct HandshakeRequestBody {
    clientId: String,
    token: String,
}

#[derive(Deserialize)]
struct HandshakeResponse {
    private: String,
    certificate: String,
    id: String,
}

macro_rules! unwrap_or_return {
    ( $e:expr, $app:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                $app.emit("auth/failed", ()).unwrap();
                return;
            }
        }
    };
}

pub fn recieve_handshake(app: AppHandle, path: String) {
    // Tell the app we're processing
    app.emit("auth/processing", ()).unwrap();

    let path_chunks: Vec<&str> = path.split("/").collect();
    if path_chunks.len() != 3 {
        app.emit("auth/failed", ()).unwrap();
        return;
    }

    let base_url = {
        let handle = DB.borrow_data().unwrap();
        Url::parse(handle.base_url.as_str()).unwrap()
    };

    let client_id = path_chunks.get(1).unwrap();
    let token = path_chunks.get(2).unwrap();
    let body = HandshakeRequestBody {
        clientId: client_id.to_string(),
        token: token.to_string(),
    };

    let endpoint = unwrap_or_return!(base_url.join("/api/v1/client/handshake"), app);
    let client = reqwest::blocking::Client::new();
    let response = unwrap_or_return!(client.post(endpoint).json(&body).send(), app);
    info!("server responded with {}", response.status());
    let response_struct = unwrap_or_return!(response.json::<HandshakeResponse>(), app);

    {
        let mut handle = DB.borrow_data_mut().unwrap();
        handle.certs = Some(DatabaseCerts {
            private: response_struct.private,
            cert: response_struct.certificate,
        });
        drop(handle);
        DB.save().unwrap();
    }

    {
        let app_state = app.state::<Mutex<AppState>>();
        let mut app_state_handle = app_state.lock().unwrap();
        app_state_handle.status = AppStatus::SignedIn;
    }

    app.emit("auth/finished", ()).unwrap();
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

    info!("opening web browser to continue authentication");
    webbrowser::open(&complete_redir_url.to_string()).unwrap();

    return Ok(());
}

pub fn setup() -> Result<(AppStatus, Option<User>), Error> {
    let data = DB.borrow_data().unwrap();

    // If we have certs, exit for now
    if data.certs.is_some() {
        // TODO: check if it's still valid, and fetch user information
        info!("have existing certs, assuming logged in...");
        return Ok((AppStatus::SignedInNeedsReauth, None));
    }

    drop(data);

    return Ok((AppStatus::SignedOut, None));
}
