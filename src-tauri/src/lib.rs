mod auth;
mod db;
mod remote;
mod unpacker;

use std::{
    io,
    sync::{LazyLock, Mutex},
    task, thread,
};
use env_logger;
use env_logger::Env;
use auth::{auth_initiate, recieve_handshake};
use db::{DatabaseInterface, DATA_ROOT_DIR};
use log::info;
use remote::{gen_drop_url, use_remote};
use serde::{Deserialize, Serialize};
use structured_logger::{json::new_writer, Builder};
use tauri_plugin_deep_link::DeepLinkExt;

#[derive(Clone, Copy, Serialize)]
pub enum AppStatus {
    NotConfigured,
    SignedOut,
    SignedIn,
    SignedInNeedsReauth,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct User {
    id: String,
    username: String,
    admin: bool,
    display_name: String,
    profile_picture: String,
}

#[derive(Clone, Serialize)]
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

fn setup() -> AppState {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let is_set_up = db::is_set_up();
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

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(|| db::setup());

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = setup();
    info!("Initialized drop client");

    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            // when defining deep link schemes at runtime, you must also check `argv` here
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            fetch_state,
            auth_initiate,
            use_remote,
            gen_drop_url,
        ])
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
                info!("registered all pre-defined deep links");
            }

            let handle = app.handle().clone();

            let main_window = tauri::WebviewWindowBuilder::new(
                &handle,
                "main", // BTW this is not the name of the window, just the label. Keep this 'main', there are permissions & configs that depend on it
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Drop Desktop App")
            .min_inner_size(900.0, 900.0)
            .inner_size(1536.0, 864.0)
            .decorations(false)
            .data_directory(DATA_ROOT_DIR.join(".webview"))
            .build()
            .unwrap();

            app.deep_link().on_open_url(move |event| {
                info!("handling drop:// url");
                let binding = event.urls();
                let url = binding.get(0).unwrap();
                if url.host_str().unwrap() == "handshake" {
                    recieve_handshake(handle.clone(), url.path().to_string())
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
