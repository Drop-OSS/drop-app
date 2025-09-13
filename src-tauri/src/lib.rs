#![deny(unused_must_use)]
#![feature(fn_traits)]
#![feature(duration_constructors)]
#![feature(duration_millis_float)]
#![feature(iterator_try_collect)]
#![deny(clippy::all)]
#![warn(unused_extern_crates)]

mod auth;
mod client;
mod database;
mod download_manager;
mod native_library;
mod process;
mod remote;
mod setup;

use crate::auth::recieve_handshake;
use crate::native_library::collection_commands::fetch_collections;
use crate::native_library::commands::{
    fetch_game, fetch_game_status, fetch_game_version_options, fetch_library, uninstall_game,
};
use crate::native_library::downloads::commands::{download_game, resume_download};
use crate::process::commands::{open_process_logs, update_game_configuration};
use crate::remote::commands::auth_initiate_code;
use crate::remote::server_proto::{handle_server_proto, handle_server_proto_offline};
use client::commands::fetch_state;
use client::{
    autostart::{get_autostart_enabled, toggle_autostart},
    cleanup::{cleanup_and_exit, quit},
};
use database::commands::{
    add_download_dir, delete_download_dir, fetch_download_dir_stats, fetch_settings,
    fetch_system_data, update_settings,
};
use download_manager::commands::{
    cancel_game, move_download_in_queue, pause_downloads, resume_downloads,
};
use drop_database::borrow_db_mut_checked;
use drop_database::db::DATA_ROOT_DIR;
use drop_database::runtime_models::User;
use drop_downloads::download_manager_frontend::DownloadManager;
use drop_process::process_manager::ProcessManager;
use drop_remote::{fetch_object::fetch_object, offline};
use log::{debug, info, warn};
use process::commands::{kill_game, launch_game};
use remote::commands::{
    auth_initiate, fetch_drop_object, gen_drop_url, manual_recieve_handshake, retry_connect,
    sign_out, use_remote,
};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use std::{env, panic};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, RunEvent, WindowEvent};
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    status: AppStatus,
    user: Option<User>,

    #[serde(skip_serializing)]
    download_manager: Arc<DownloadManager>,
    #[serde(skip_serializing)]
    process_manager: &'static Mutex<ProcessManager<'static>>,
}

pub fn custom_panic_handler(e: &PanicHookInfo) -> Option<()> {
    let crash_file = DATA_ROOT_DIR.join(format!(
        "crash-{}.log",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs()
    ));
    let mut file = File::create_new(crash_file).ok()?;
    file.write_all(format!("Drop crashed with the following panic:\n{e}").as_bytes())
        .ok()?;
    drop(file);

    Some(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    panic::set_hook(Box::new(|e| {
        let _ = custom_panic_handler(e);
        println!("{e}");
    }));

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            auth_initiate_code,
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
            fetch_game_version_options,
            update_game_configuration,
            // Collections
            fetch_collections,
            // fetch_collection,
            // create_collection,
            // add_game_to_collection,
            // delete_collection,
            // delete_game_in_collection,
            // Downloads
            download_game,
            resume_download,
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
            open_process_logs
        ])
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimize"]),
        ))
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                let state = setup::setup(handle).await;
                info!("initialized drop client");
                app.manage(Mutex::new(state));

                {
                    use tauri_plugin_deep_link::DeepLinkExt;
                    let _ = app.deep_link().register_all();
                    debug!("registered all pre-defined deep links");
                }

                let handle = app.handle().clone();

                let _main_window = tauri::WebviewWindowBuilder::new(
                    &handle,
                    "main", // BTW this is not the name of the window, just the label. Keep this 'main', there are permissions & configs that depend on it
                    tauri::WebviewUrl::App("main".into()),
                )
                .title("Drop Desktop App")
                .min_inner_size(1000.0, 500.0)
                .inner_size(1536.0, 864.0)
                .decorations(false)
                .shadow(false)
                .data_directory(DATA_ROOT_DIR.join(".webview"))
                .build()
                .unwrap();

                app.deep_link().on_open_url(move |event| {
                    debug!("handling drop:// url");
                    let binding = event.urls();
                    let url = binding.first().unwrap();
                    if url.host_str().unwrap() == "handshake" {
                        tauri::async_runtime::spawn(recieve_handshake(
                            handle.clone(),
                            url.path().to_string(),
                        ));
                    }
                });

                let menu = Menu::with_items(
                    app,
                    &[
                        &MenuItem::with_id(app, "open", "Open", true, None::<&str>).unwrap(),
                        &PredefinedMenuItem::separator(app).unwrap(),
                        /*
                        &MenuItem::with_id(app, "show_library", "Library", true, None::<&str>)?,
                        &MenuItem::with_id(app, "show_settings", "Settings", true, None::<&str>)?,
                        &PredefinedMenuItem::separator(app)?,
                         */
                        &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap(),
                    ],
                )
                .unwrap();

                run_on_tray(|| {
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
                });

                {
                    let mut db_handle = borrow_db_mut_checked();
                    if let Some(original) = db_handle.prev_database.take() {
                        warn!(
                            "Database corrupted. Original file at {}",
                            original.canonicalize().unwrap().to_string_lossy()
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
            });

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("object", move |_ctx, request, responder| {
            tauri::async_runtime::spawn(async move {
                fetch_object(request, responder).await;
            });
        })
        .register_asynchronous_uri_scheme_protocol("server", |ctx, request, responder| {
            tauri::async_runtime::block_on(async move {
                let state = ctx
                    .app_handle()
                    .state::<tauri::State<'_, Mutex<AppState>>>();

                offline!(
                    state,
                    handle_server_proto,
                    handle_server_proto_offline,
                    request,
                    responder
                )
                .await;
            });
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                run_on_tray(|| {
                    window.hide().unwrap();
                    api.prevent_close();
                });
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, event| {
        if let RunEvent::ExitRequested { code, api, .. } = event {
            run_on_tray(|| {
                if code.is_none() {
                    api.prevent_exit();
                }
            });
        }
    });
}

fn run_on_tray<T: FnOnce()>(f: T) {
    if match std::env::var("NO_TRAY_ICON") {
        Ok(s) => s.to_lowercase() != "true",
        Err(_) => true,
    } {
        (f)();
    }
}
