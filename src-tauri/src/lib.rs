mod database;
mod games;

mod autostart;
mod cleanup;
mod commands;
mod download_manager;
mod error;
mod process;
mod remote;

use crate::database::db::DatabaseImpls;
use autostart::{get_autostart_enabled, toggle_autostart};
use cleanup::{cleanup_and_exit, quit};
use commands::fetch_state;
use database::commands::{
    add_download_dir, delete_download_dir, fetch_download_dir_stats, fetch_settings,
    fetch_system_data, update_settings,
};
use database::db::{
    borrow_db_checked, borrow_db_mut_checked, DatabaseInterface, GameDownloadStatus, DATA_ROOT_DIR,
};
use download_manager::commands::{
    cancel_game, move_download_in_queue, pause_downloads, resume_downloads,
};
use download_manager::download_manager::DownloadManager;
use download_manager::download_manager_builder::DownloadManagerBuilder;
use games::commands::{
    fetch_game, fetch_game_status, fetch_game_verion_options, fetch_library, uninstall_game,
};
use games::downloads::commands::download_game;
use games::library::Game;
use http::Response;
use http::{header::*, response::Builder as ResponseBuilder};
use log::{debug, info, warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use process::commands::{kill_game, launch_game};
use process::process_manager::ProcessManager;
use remote::auth::{self, generate_authorization_header, recieve_handshake};
use remote::commands::{
    auth_initiate, fetch_drop_object, gen_drop_url, manual_recieve_handshake, retry_connect, sign_out, use_remote
};
use remote::requests::make_request;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;

#[derive(Clone, Copy, Serialize, Eq, PartialEq)]
pub enum AppStatus {
    NotConfigured,
    Offline,
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
pub struct AppState<'a> {
    status: AppStatus,
    user: Option<User>,
    games: HashMap<String, Game>,

    #[serde(skip_serializing)]
    download_manager: Arc<DownloadManager>,
    #[serde(skip_serializing)]
    process_manager: Arc<Mutex<ProcessManager<'a>>>,
}

fn setup(handle: AppHandle) -> AppState<'static> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d} | {l} | {f}:{L} - {m}{n}",
        )))
        .append(false)
        .build(DATA_ROOT_DIR.lock().unwrap().join("./drop.log"))
        .unwrap();

    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d} | {l} | {f}:{L} - {m}{n}",
        )))
        .build();

    let log_level = env::var("RUST_LOG").unwrap_or(String::from("Info"));

    let config = Config::builder()
        .appenders(vec![
            Appender::builder().build("logfile", Box::new(logfile)),
            Appender::builder().build("console", Box::new(console)),
        ])
        .build(
            Root::builder()
                .appenders(vec!["logfile", "console"])
                .build(LevelFilter::from_str(&log_level).expect("Invalid log level")),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    let games = HashMap::new();
    let download_manager = Arc::new(DownloadManagerBuilder::build(handle.clone()));
    let process_manager = Arc::new(Mutex::new(ProcessManager::new(handle.clone())));

    debug!("checking if database is set up");
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

    debug!("database is set up");

    // TODO: Account for possible failure
    let (app_status, user) = auth::setup();

    let db_handle = borrow_db_checked();
    let mut missing_games = Vec::new();
    let statuses = db_handle.applications.game_statuses.clone();
    drop(db_handle);
    for (game_id, status) in statuses.into_iter() {
        match status {
            database::db::GameDownloadStatus::Remote {} => {}
            database::db::GameDownloadStatus::SetupRequired {
                version_name: _,
                install_dir,
            } => {
                let install_dir_path = Path::new(&install_dir);
                if !install_dir_path.exists() {
                    missing_games.push(game_id);
                }
            }
            database::db::GameDownloadStatus::Installed {
                version_name: _,
                install_dir,
            } => {
                let install_dir_path = Path::new(&install_dir);
                if !install_dir_path.exists() {
                    missing_games.push(game_id);
                }
            }
        }
    }

    info!("detected games missing: {:?}", missing_games);

    let mut db_handle = borrow_db_mut_checked();
    for game_id in missing_games {
        db_handle
            .applications
            .game_statuses
            .entry(game_id)
            .and_modify(|v| *v = GameDownloadStatus::Remote {});
    }

    drop(db_handle);

    debug!("finished setup!");

    // Sync autostart state
    if let Err(e) = autostart::sync_autostart_on_startup(&handle) {
        warn!("failed to sync autostart state: {}", e);
    }

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
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    #[allow(unused_variables)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            // when defining deep link schemes at runtime, you must also check `argv` here
        }));
    }

    let app = builder
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            // Core utils
            fetch_state,
            quit,
            fetch_system_data,
            // User utils
            update_settings,
            fetch_settings,
            // Auth
            auth_initiate,
            retry_connect,
            manual_recieve_handshake,
            sign_out,
            // Remote
            use_remote,
            gen_drop_url,
            fetch_drop_object,
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
            move_download_in_queue,
            pause_downloads,
            resume_downloads,
            cancel_game,
            uninstall_game,
            // Processes
            launch_game,
            kill_game,
            toggle_autostart,
            get_autostart_enabled,
        ])
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimize"]),
        ))
        .setup(|app| {
            let handle = app.handle().clone();
            let state = setup(handle);
            debug!("initialized drop client");
            app.manage(Mutex::new(state));

            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
                debug!("registered all pre-defined deep links");
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
            .shadow(false)
            .data_directory(DATA_ROOT_DIR.lock().unwrap().join(".webview"))
            .build()
            .unwrap();

            app.deep_link().on_open_url(move |event| {
                debug!("handling drop:// url");
                let binding = event.urls();
                let url = binding.first().unwrap();
                if url.host_str().unwrap() == "handshake" {
                    recieve_handshake(handle.clone(), url.path().to_string())
                }
            });

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "open", "Open", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    /*
                    &MenuItem::with_id(app, "show_library", "Library", true, None::<&str>)?,
                    &MenuItem::with_id(app, "show_settings", "Settings", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                     */
                    &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                ],
            )?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        app.webview_windows().get("main").unwrap().show().unwrap();
                    }
                    "quit" => {
                        cleanup_and_exit(app, &app.state());
                    }

                    _ => {
                        warn!("menu event not handled: {:?}", event.id);
                    }
                })
                .build(app)
                .expect("error while setting up tray menu");

            {
                let mut db_handle = borrow_db_mut_checked();
                if let Some(original) = db_handle.prev_database.take() {
                    warn!(
                        "Database corrupted. Original file at {}",
                        original
                            .canonicalize()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    );
                    app.dialog()
                        .message(
                            "Database corrupted. A copy has been saved at: ".to_string()
                                + original.to_str().unwrap(),
                        )
                        .title("Database corrupted")
                        .show(|_| {});
                }
            }

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("object", move |_ctx, request, responder| {
            // Drop leading /
            let object_id = &request.uri().path()[1..];

            let header = generate_authorization_header();
            let client: reqwest::blocking::Client = reqwest::blocking::Client::new();
            let response = make_request(&client, &["/api/v1/client/object/", object_id], &[], |f| {
                f.header("Authorization", header)
            })
            .unwrap()
            .send();
            if response.is_err() {
                warn!(
                    "failed to fetch object with error: {}",
                    response.err().unwrap()
                );
                responder.respond(Response::builder().status(500).body(Vec::new()).unwrap());
                return;
            }
            let response = response.unwrap();

            let resp_builder = ResponseBuilder::new().header(
                CONTENT_TYPE,
                response.headers().get("Content-Type").unwrap(),
            );
            let data = Vec::from(response.bytes().unwrap());
            let resp = resp_builder.body(data).unwrap();

            responder.respond(resp);
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, event| {
        if let RunEvent::ExitRequested { code, api, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
