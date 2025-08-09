use std::sync::Mutex;

use futures_lite::StreamExt;
use log::{debug, warn};
use reqwest_websocket::{Message, RequestBuilderExt};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{
    AppState, AppStatus,
    database::db::{borrow_db_checked, borrow_db_mut_checked},
    error::remote_access_error::RemoteAccessError,
    remote::{
        auth::generate_authorization_header,
        requests::generate_url,
        utils::{DROP_CLIENT_SYNC, DROP_CLIENT_WS_CLIENT},
    },
};

use super::{
    auth::{auth_initiate_logic, recieve_handshake, setup},
    cache::{cache_object, get_cached_object},
    utils::use_remote_logic,
};

#[tauri::command]
pub async fn use_remote(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), RemoteAccessError> {
    use_remote_logic(url, state).await
}

#[tauri::command]
pub fn gen_drop_url(path: String) -> Result<String, RemoteAccessError> {
    let base_url = {
        let handle = borrow_db_checked();

        Url::parse(&handle.base_url).map_err(RemoteAccessError::ParsingError)?
    };

    let url = base_url.join(&path).unwrap();

    Ok(url.to_string())
}

#[tauri::command]
pub fn fetch_drop_object(path: String) -> Result<Vec<u8>, RemoteAccessError> {
    let _drop_url = gen_drop_url(path.clone())?;
    let req = generate_url(&[&path], &[])?;
    let req = DROP_CLIENT_SYNC
        .get(req)
        .header("Authorization", generate_authorization_header())
        .send();

    match req {
        Ok(data) => {
            let data = data.bytes()?.to_vec();
            cache_object(&path, &data)?;
            Ok(data)
        }
        Err(e) => {
            debug!("{e}");
            get_cached_object::<Vec<u8>>(&path)
        }
    }
}
#[tauri::command]
pub fn sign_out(app: AppHandle) {
    // Clear auth from database
    {
        let mut handle = borrow_db_mut_checked();
        handle.auth = None;
    }

    // Update app state
    {
        let app_state = app.state::<Mutex<AppState>>();
        let mut app_state_handle = app_state.lock().unwrap();
        app_state_handle.status = AppStatus::SignedOut;
        app_state_handle.user = None;
    }

    // Emit event for frontend
    app.emit("auth/signedout", ()).unwrap();
}

#[tauri::command]
pub async fn retry_connect(state: tauri::State<'_, Mutex<AppState<'_>>>) -> Result<(), ()> {
    let (app_status, user) = setup().await;

    let mut guard = state.lock().unwrap();
    guard.status = app_status;
    guard.user = user;
    drop(guard);

    Ok(())
}

#[tauri::command]
pub fn auth_initiate() -> Result<(), RemoteAccessError> {
    let base_url = {
        let db_lock = borrow_db_checked();
        Url::parse(&db_lock.base_url.clone())?
    };

    let redir_url = auth_initiate_logic("callback".to_string())?;
    let complete_redir_url = base_url.join(&redir_url)?;

    debug!("opening web browser to continue authentication");
    webbrowser::open(complete_redir_url.as_ref()).unwrap();
    Ok(())
}

#[derive(Deserialize)]
struct CodeWebsocketResponse {
    #[serde(rename = "type")]
    response_type: String,
    value: String,
}

#[tauri::command]
pub fn auth_initiate_code(app: AppHandle) -> Result<String, RemoteAccessError> {
    let base_url = {
        let db_lock = borrow_db_checked();
        Url::parse(&db_lock.base_url.clone())?
    };

    let code = auth_initiate_logic("code".to_string())?;
    let header_code = code.clone();

    println!("using code: {} to sign in", code);

    tauri::async_runtime::spawn(async move {
        let load = async || -> Result<(), RemoteAccessError> {
            let ws_url = base_url.join("/api/v1/client/auth/code/ws")?;
            let response = DROP_CLIENT_WS_CLIENT
                .get(ws_url)
                .header("Authorization", header_code)
                .upgrade()
                .send()
                .await?;

            let mut websocket = response.into_websocket().await?;

            while let Some(token) = websocket.try_next().await? {
                if let Message::Text(response) = token {
                    let response = serde_json::from_str::<CodeWebsocketResponse>(&response)
                        .map_err(|e| RemoteAccessError::UnparseableResponse(e.to_string()))?;
                    match response.response_type.as_str() {
                        "token" => {
                            let recieve_app = app.clone();
                            manual_recieve_handshake(recieve_app, response.value).await.unwrap();
                            return Ok(());
                        }
                        _ => return Err(RemoteAccessError::HandshakeFailed(response.value)),
                    }
                }
            }

            Err(RemoteAccessError::HandshakeFailed(
                "Failed to connect to websocket".to_string(),
            ))
        };

        let result = load().await;
        if let Err(err) = result {
            warn!("{err}");
            app.emit("auth/failed", err.to_string()).unwrap();
        }
    });

    Ok(code)
}

#[tauri::command]
pub async fn manual_recieve_handshake(app: AppHandle, token: String) -> Result<(), ()> {
    recieve_handshake(app, format!("handshake/{token}")).await;

    Ok(())
}
