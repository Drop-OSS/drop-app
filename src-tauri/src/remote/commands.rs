use log::debug;
use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{
    AppStatus, DropFunctionState,
    database::db::{borrow_db_checked, borrow_db_mut_checked},
    error::remote_access_error::RemoteAccessError,
    remote::{auth::generate_authorization_header, requests::make_request},
};

use super::{
    auth::{auth_initiate_logic, recieve_handshake, setup},
    cache::{cache_object, get_cached_object},
    utils::use_remote_logic,
};

#[tauri::command]
pub async fn use_remote(
    url: String,
    state: tauri::State<'_, DropFunctionState<'_>>,
) -> Result<(), RemoteAccessError> {
    use_remote_logic(url, state).await
}

#[tauri::command]
pub async fn gen_drop_url(path: String) -> Result<String, RemoteAccessError> {
    let base_url = {
        let handle = borrow_db_checked().await;

        Url::parse(&handle.base_url).map_err(RemoteAccessError::ParsingError)?
    };

    let url = base_url.join(&path).unwrap();

    Ok(url.to_string())
}

#[tauri::command]
pub async fn fetch_drop_object(path: String) -> Result<Vec<u8>, RemoteAccessError> {
    let _drop_url = gen_drop_url(path.clone()).await?;
    let req = make_request(&Client::new(), &[&path], &[], async |r| {
        r.header("Authorization", generate_authorization_header().await)
    })
    .await?
    .send()
    .await;

    match req {
        Ok(data) => {
            let data = data.bytes().await?.to_vec();
            cache_object(&path, &data).await?;
            Ok(data)
        }
        Err(e) => {
            debug!("{e}");
            get_cached_object::<&str, Vec<u8>>(&path).await
        }
    }
}
#[tauri::command]
pub async fn sign_out(app: AppHandle) {
    // Clear auth from database
    {
        let mut handle = borrow_db_mut_checked().await;
        handle.auth = None;
    }

    // Update app state
    {
        let app_state = app.state::<DropFunctionState<'_>>();
        let mut app_state_handle = app_state.lock().await;
        app_state_handle.status = AppStatus::SignedOut;
        app_state_handle.user = None;
    }

    // Emit event for frontend
    app.emit("auth/signedout", ()).unwrap();
}

#[tauri::command]
pub async fn retry_connect(state: tauri::State<'_, DropFunctionState<'_>>) -> Result<(), ()> {
    let (app_status, user) = setup().await;

    let mut guard = state.lock().await;
    guard.status = app_status;
    guard.user = user;
    drop(guard);

    Ok(())
}

#[tauri::command]
pub async fn auth_initiate() -> Result<(), RemoteAccessError> {
    auth_initiate_logic().await
}

#[tauri::command]
pub async fn manual_recieve_handshake(app: AppHandle, token: String) {
    recieve_handshake(app, format!("handshake/{token}")).await;
}
