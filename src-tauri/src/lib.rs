mod auth;
mod data;
mod remote;
mod unpacker;

use std::{
    sync::{LazyLock, Mutex},
    task, thread,
};

use auth::{auth_initiate, recieve_handshake};
use data::DatabaseInterface;
use futures::executor;
use remote::use_remote;
use serde::Serialize;
use tauri_plugin_deep_link::DeepLinkExt;

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

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
          println!("a new app instance was opened with {argv:?} and the deep link event was already triggered");
          // when defining deep link schemes at runtime, you must also check `argv` here
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            fetch_state,
            auth_initiate,
            use_remote
        ])
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }

            let handle = app.handle().clone();

            app.deep_link().on_open_url(move |event| {
                let binding = event.urls();
                let url = binding.get(0).unwrap();
                match url.host_str().unwrap() {
                    "handshake" => {
                        executor::block_on(recieve_handshake(
                            handle.clone(),
                            url.path().to_string(),
                        ));
                    }
                    _ => (),
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
