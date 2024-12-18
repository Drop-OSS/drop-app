mod auth;
mod db;
mod downloads;
mod library;

mod process;
mod remote;
mod settings;
#[cfg(test)]
mod tests;

use crate::db::DatabaseImpls;
use auth::{auth_initiate, generate_authorization_header, recieve_handshake, retry_connect};
use db::{
    add_download_dir, delete_download_dir, fetch_download_dir_stats, DatabaseInterface,
    DATA_ROOT_DIR,
};
use downloads::download_commands::*;
use downloads::download_manager::DownloadManager;
use downloads::download_manager_builder::DownloadManagerBuilder;
use http::{header::*, response::Builder as ResponseBuilder};
use library::{fetch_game, fetch_game_status, fetch_game_verion_options, fetch_library, Game};
use log::{debug, info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use process::process_commands::launch_game;
use process::process_manager::ProcessManager;
use remote::{gen_drop_url, use_remote};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use tauri::menu::{Menu, MenuItem, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

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
    download_manager: Arc<DownloadManager>,
    #[serde(skip_serializing)]
    process_manager: Arc<Mutex<ProcessManager>>,
}

#[tauri::command]
fn fetch_state(state: tauri::State<'_, Mutex<AppState>>) -> Result<AppState, String> {
    let guard = state.lock().unwrap();
    let cloned_state = guard.clone();
    drop(guard);
    Ok(cloned_state)
}

fn setup(handle: AppHandle) -> AppState {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} | {l} | {f} - {m}{n}")))
        .build(DATA_ROOT_DIR.lock().unwrap().join("./drop.log"))
        .unwrap();

    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{t}|{l}|{f} - {m}{n}")))
        .build();

    let config = Config::builder()
        .appenders(vec![
            Appender::builder().build("logfile", Box::new(logfile)),
            Appender::builder().build("console", Box::new(console)),
        ])
        .build(
            Root::builder()
                .appenders(vec!["logfile", "console"])
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    let games = HashMap::new();
    let download_manager = Arc::new(DownloadManagerBuilder::build(handle));
    let process_manager = Arc::new(Mutex::new(ProcessManager::new()));

    debug!("Checking if database is set up");
    let is_set_up = DB.database_is_set_up();
    if !is_set_up {
        return AppState {
            status: AppStatus::NotConfigured,
            user: None,
            games,
            download_manager,
            process_manager,
        };
    }

    debug!("Database is set up");

    let (app_status, user) = auth::setup().unwrap();
    AppState {
        status: app_status,
        user,
        games,
        download_manager,
        process_manager,
    }
}

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(DatabaseInterface::set_up_database);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    #[allow(unused_variables)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            // when defining deep link schemes at runtime, you must also check `argv` here
        }));
    }

    let mut app = builder
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            // DB
            fetch_state,
            // Auth
            auth_initiate,
            retry_connect,
            // Remote
            use_remote,
            gen_drop_url,
            // Library
            fetch_library,
            fetch_game,
            add_download_dir,
            delete_download_dir,
            fetch_download_dir_stats,
            fetch_game_status,
            fetch_game_verion_options,
            // Downloads
            download_game,
            move_game_in_queue,
            pause_game_downloads,
            resume_game_downloads,
            // Processes
            launch_game,
        ])
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = setup(handle);
            info!("initialized drop client");
            app.manage(Mutex::new(state));

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
            .min_inner_size(1000.0, 500.0)
            .inner_size(1536.0, 864.0)
            .decorations(false)
            .data_directory(DATA_ROOT_DIR.lock().unwrap().join(".webview"))
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

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show_library", "Library", true, None::<&str>)?,
                    &MenuItem::with_id(app, "show_settings", "Settings", true, None::<&str>)?,
                    &MenuItem::with_id(app, "open", "Open", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                ],
            )?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .build(app)
                .expect("error while setting up tray menu");

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
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, e| match e {
        _ => {}
    });

    info!("exiting drop application...");
}
