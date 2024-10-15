mod auth;
mod db;
mod library;
mod remote;
mod unpacker;

use auth::{auth_initiate, generate_authorization_header, recieve_handshake};
use db::{fetch_base_url, DatabaseInterface, DATA_ROOT_DIR};
use env_logger;
use env_logger::Env;
use http::{header::*, response::Builder as ResponseBuilder, status::StatusCode};
use library::{fetch_game, fetch_library, Game};
use log::info;
use remote::{gen_drop_url, use_remote};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, io, sync::{LazyLock, Mutex}, task, thread
};
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
pub struct User {
    id: String,
    username: String,
    admin: bool,
    displayName: String,
    profilePicture: String,
}

#[derive(Clone, Serialize)]
pub struct AppState {
    status: AppStatus,
    user: Option<User>,
    games: HashMap<String, Game>,
}

#[tauri::command]
fn fetch_state<'a>(state: tauri::State<'_, Mutex<AppState>>) -> Result<AppState, String> {
    let guard = state.lock().unwrap();
    let cloned_state = guard.clone();
    drop(guard);
    Ok(cloned_state)
}

fn setup<'a>() -> AppState {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let is_set_up = db::is_set_up();
    if !is_set_up {
        return AppState {
            status: AppStatus::NotConfigured,
            user: None,
            games: HashMap::new(),
        };
    }

    let auth_result = auth::setup().unwrap();
    return AppState {
        status: auth_result.0,
        user: auth_result.1,
        games: HashMap::new(),
    };
}

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(|| db::setup());

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
                let url = binding.get(0).unwrap();
                match url.host_str().unwrap() {
                    "handshake" => recieve_handshake(handle.clone(), url.path().to_string()),
                    _ => (),
                }
            });

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("object", move |_ctx, request, responder| {
            let base_url = fetch_base_url();

            // Drop leading /
            let object_id = &request.uri().path()[1..];

            let object_url = base_url
                .join("/api/v1/client/object/")
                .unwrap()
                .join(object_id)
                .unwrap();

            info!["{}", object_url.to_string()];

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
