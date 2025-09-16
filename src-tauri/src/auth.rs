use std::sync::Mutex;

use drop_database::{borrow_db_checked, runtime_models::User};
use drop_errors::remote_access_error::RemoteAccessError;
use drop_remote::{auth::{fetch_user, recieve_handshake_logic}, cache::{cache_object, clear_cached_object, get_cached_object}};
use log::warn;
use tauri::{AppHandle, Emitter as _, Manager as _};

use crate::{AppState, AppStatus};

pub async fn setup() -> (AppStatus, Option<User>) {
    let auth = {
        let data = borrow_db_checked();
        data.auth.clone()
    };

    if auth.is_some() {
        let user_result = match fetch_user().await {
            Ok(data) => data,
            Err(RemoteAccessError::FetchError(_)) => {
                let user = get_cached_object::<User>("user").unwrap();
                return (AppStatus::Offline, Some(user));
            }
            Err(_) => return (AppStatus::SignedInNeedsReauth, None),
        };
        cache_object("user", &user_result).unwrap();
        return (AppStatus::SignedIn, Some(user_result));
    }

    (AppStatus::SignedOut, None)
}

pub async fn recieve_handshake(app: AppHandle, path: String) {
    // Tell the app we're processing
    app.emit("auth/processing", ()).unwrap();

    let handshake_result = recieve_handshake_logic(path).await;
    if let Err(e) = handshake_result {
        warn!("error with authentication: {e}");
        app.emit("auth/failed", e.to_string()).unwrap();
        return;
    }

    let app_state = app.state::<Mutex<AppState>>();

    let (app_status, user) = setup().await;

    let mut state_lock = app_state.lock().unwrap();

    state_lock.status = app_status;
    state_lock.user = user;

    let _ = clear_cached_object("collections");
    let _ = clear_cached_object("library");

    drop(state_lock);

    app.emit("auth/finished", ()).unwrap();
}
