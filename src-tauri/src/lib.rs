mod auth;
mod db;
mod library;
mod remote;
mod downloads;
#[cfg(test)]
mod tests;

use crate::db::DatabaseImpls;
use auth::{auth_initiate, generate_authorization_header, recieve_handshake};
use db::{DatabaseInterface, DATA_ROOT_DIR};
use downloads::download_commands::{queue_game_download, start_game_downloads, stop_specific_game_download};
use env_logger::Env;
use http::{header::*, response::Builder as ResponseBuilder};
use library::{fetch_game, fetch_library, Game};
use log::info;
use remote::{gen_drop_url, use_remote};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use std::sync::Arc;
use tauri_plugin_deep_link::DeepLinkExt;
use crate::db::DatabaseImpls;
use crate::downloads::download_agent::{GameDownloadAgent};

#[derive(Clone, Copy, Serialize)]
pub enum AppStatus {
    NotConfigured,
    ServerError,
    SignedOut,
    SignedIn,
    SignedInNeedsReauth,
    ServerUnavailable,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: String,
    username: String,
    admin: bool,
    display_name: String,
    profile_picture: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    status: AppStatus,
    user: Option<User>,
    games: HashMap<String, Game>,
    
    #[serde(skip_serializing)]
    game_downloads: HashMap<String, Arc<GameDownloadAgent>>
}

#[tauri::command]
fn fetch_state(state: tauri::State<'_, Mutex<AppState>>) -> Result<AppState, String> {
    let guard = state.lock().unwrap();
    let cloned_state = guard.clone();
    drop(guard);
    Ok(cloned_state)
}

fn setup() -> AppState {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let is_set_up = DB.database_is_set_up();
    if !is_set_up {
        return AppState {
            status: AppStatus::NotConfigured,
            user: None,
            games: HashMap::new(),
            game_downloads: HashMap::new(),
        };
    }

    let (app_status, user) = auth::setup().unwrap();
    AppState {
        status: app_status,
        user: user,
        games: HashMap::new(),
        game_downloads: HashMap::new(),
    }
}

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(DatabaseInterface::set_up_database);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = setup();
    info!("initialized drop client");

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
            // DB
            fetch_state,
            // Auth
            auth_initiate,
            // Remote
            use_remote,
            gen_drop_url,
            // Library
            fetch_library,
            fetch_game,
            // Downloads
            queue_game_download,
            start_game_downloads,
            stop_specific_game_download
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

            let _main_window = tauri::WebviewWindowBuilder::new(
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
                let url = binding.first().unwrap();
                if url.host_str().unwrap() == "handshake" {
                    recieve_handshake(handle.clone(), url.path().to_string())
                }
            });

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("object", move |_ctx, request, responder| {
            let base_url = DB.fetch_base_url();

            // Drop leading /
            let object_id = &request.uri().path()[1..];

            let object_url = base_url
                .join("/api/v1/client/object/")
                .unwrap()
                .join(object_id)
                .unwrap();

            let header = generate_authorization_header();
            let client: reqwest::blocking::Client = reqwest::blocking::Client::new();
            let response = client
                .get(object_url.to_string())
                .header("Authorization", header)
                .send()
                .unwrap();

            let resp_builder = ResponseBuilder::new().header(
                CONTENT_TYPE,
                response.headers().get("Content-Type").unwrap(),
            );
            let data = Vec::from(response.bytes().unwrap());
            let resp = resp_builder.body(data).unwrap();

            responder.respond(resp);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
