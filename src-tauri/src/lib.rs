mod auth;
mod data;
mod unpacker;

use data::DatabaseInterface;
use serde::Serialize;
use tauri::Runtime;

#[derive(Clone, Copy, Serialize)]
pub enum AppAuthenticationStatus {
    SignedOut,
    SignedIn,
    SignedInNeedsReauth,
}
#[derive(Clone, Copy, Serialize)]
pub struct User {}

#[derive(Clone, Copy, Serialize)]
pub struct AppState {
    auth: AppAuthenticationStatus,
    user: Option<User>,
}

#[tauri::command]
fn fetch_state(state: tauri::State<AppState>) -> Result<AppState, String> {
    Ok(*state.inner())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db: DatabaseInterface = data::setup();
    let auth_result = auth::setup(db).unwrap();

    let state = AppState {
        auth: auth_result.0,
        user: auth_result.1,
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![fetch_state])
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
