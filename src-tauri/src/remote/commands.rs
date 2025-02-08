use std::sync::Mutex;

use log::debug;
use reqwest::blocking::Client;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{
    database::db::{borrow_db_checked, borrow_db_mut_checked, save_db}, error::remote_access_error::RemoteAccessError, remote::{auth::generate_authorization_header, requests::make_request}, AppState, AppStatus
};

use super::{
    auth::{auth_initiate_logic, recieve_handshake, setup}, cache::{cache_object, get_cached_object}, remote::use_remote_logic
};

#[tauri::command]
pub fn use_remote(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), RemoteAccessError> {
    use_remote_logic(url, state)
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
    let drop_url = gen_drop_url(path.clone());
    let req = make_request(
        &Client::new(), 
        &[&path],
        &[], 
        |r| { r.header("Authorization", generate_authorization_header()) }
    )?.send();

    match req {
        Ok(data) => {
            let data = data.bytes()?.to_vec();
            cache_object(&path, &data)?;
            Ok(data)
        },
        Err(e) => {
            debug!("{}", e);
            get_cached_object::<&str, Vec<u8>>(&path)
        },
    }
}
#[tauri::command]
pub fn sign_out(app: AppHandle) {
    // Clear auth from database
    {
        let mut handle = borrow_db_mut_checked();
        handle.auth = None;
        drop(handle);
        save_db();
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
pub fn retry_connect(state: tauri::State<'_, Mutex<AppState>>) {
    let (app_status, user) = setup();

    let mut guard = state.lock().unwrap();
    guard.status = app_status;
    guard.user = user;
    drop(guard);
}

#[tauri::command]
pub fn auth_initiate() -> Result<(), RemoteAccessError> {
    auth_initiate_logic()
}

#[tauri::command]
pub fn manual_recieve_handshake(app: AppHandle, token: String) {
    recieve_handshake(app, format!("handshake/{}", token));
}
