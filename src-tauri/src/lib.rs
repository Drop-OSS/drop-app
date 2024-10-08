mod auth;
mod data;
mod remote;
mod unpacker;

use std::{
    borrow::Borrow,
    sync::{LazyLock, Mutex},
};

use auth::auth_initiate;
use data::DatabaseInterface;
use remote::use_remote;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub enum AppStatus {
    NotConfigured,
    SignedOut,
    SignedIn,
    SignedInNeedsReauth,
}
#[derive(Clone, Copy, Serialize)]
pub struct User {}

#[derive(Clone, Copy, Serialize)]
pub struct AppState {
    status: AppStatus,
    user: Option<User>,
}

#[tauri::command]
fn fetch_state<'a>(state: tauri::State<'_, Mutex<AppState>>) -> Result<AppState, String> {
    let guard = state.lock().unwrap();
    let cloned_state = guard.clone();
    drop(guard);
    Ok(cloned_state)
}

fn setup<'a>() -> AppState {
    let is_set_up = data::is_set_up();
    if !is_set_up {
        return AppState {
            status: AppStatus::NotConfigured,
            user: None,
        };
    }

    let auth_result = auth::setup().unwrap();
    return AppState {
        status: auth_result.0,
        user: auth_result.1,
    };
}

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(|| data::setup());

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = setup();

    tauri::Builder::default()
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            fetch_state,
            auth_initiate,
            use_remote
        ])
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
