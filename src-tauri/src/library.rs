use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use urlencoding::encode;

use crate::db::DatabaseImpls;
use crate::db::ApplicationVersion;
use crate::db::ApplicationStatus;
use crate::download_manager::download_manager::{DownloadManagerStatus, DownloadStatus};
use crate::download_manager::downloadable_metadata::DownloadableMetadata;
use crate::process::process_manager::Platform;
use crate::remote::RemoteAccessError;
use crate::state::{DownloadStatusManager, ApplicationStatusWithTransient};
use crate::{auth::generate_authorization_header, AppState, DB};

#[derive(serde::Serialize)]
pub struct FetchGameStruct {
    game: Game,
    status: ApplicationStatusWithTransient,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    meta: DownloadableMetadata,
    m_name: String,
    m_short_description: String,
    m_description: String,
    // mDevelopers
    // mPublishers
    m_icon_id: String,
    m_banner_id: String,
    m_cover_id: String,
    m_image_library: Vec<String>,
}
#[derive(serde::Serialize, Clone)]
pub struct GameUpdateEvent {
    pub game_id: String,
    pub status: ApplicationStatusWithTransient,
}

#[derive(Serialize, Clone)]
pub struct QueueUpdateEventQueueData {
    pub meta: DownloadableMetadata,
    pub status: DownloadStatus,
    pub progress: f64,
}

#[derive(serde::Serialize, Clone)]
pub struct QueueUpdateEvent {
    pub queue: Vec<QueueUpdateEventQueueData>,
    pub status: DownloadManagerStatus,
}

#[derive(serde::Serialize, Clone)]
pub struct StatsUpdateEvent {
    pub speed: usize,
    pub time: usize,
}

// Game version with some fields missing and size information
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersionOption {
    version_index: usize,
    version_name: String,
    platform: Platform,
    setup_command: String,
    launch_command: String,
    delta: bool,
    umu_id_override: Option<String>,
    // total_size: usize,
}

fn fetch_library_logic(app: AppHandle) -> Result<Vec<Game>, RemoteAccessError> {
    let base_url = DB.fetch_base_url();
    let library_url = base_url.join("/api/v1/client/user/library")?;

    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(library_url.to_string())
        .header("Authorization", header)
        .send()?;

    if response.status() != 200 {
        return Err(response.status().as_u16().into());
    }

    let games: Vec<Game> = response.json::<Vec<Game>>()?;

    let state = app.state::<Mutex<AppState>>();
    let mut handle = state.lock().unwrap();

    let mut db_handle = DB.borrow_data_mut().unwrap();

    for game in games.iter() {
        handle.games.insert(game.meta.clone(), game.clone());
        if !db_handle.applications.statuses.contains_key(&game.meta) {
            db_handle
                .applications
                .statuses
                .insert(game.meta.clone(), ApplicationStatus::Remote {});
        }
    }

    drop(handle);

    Ok(games)
}

#[tauri::command]
pub fn fetch_library(app: AppHandle) -> Result<Vec<Game>, String> {
    fetch_library_logic(app).map_err(|e| e.to_string())
}

fn fetch_game_logic(
    meta: DownloadableMetadata,
    app: tauri::AppHandle,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let state = app.state::<Mutex<AppState>>();
    let mut state_handle = state.lock().unwrap();

    let game = state_handle.games.get(&meta);
    if let Some(game) = game {
        let status = DownloadStatusManager::fetch_state(&meta);

        let data = FetchGameStruct {
            game: game.clone(),
            status,
        };

        return Ok(data);
    }

    let base_url = DB.fetch_base_url();

    let endpoint = base_url.join(&format!("/api/v1/game/{}", meta.id))?;
    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(endpoint.to_string())
        .header("Authorization", header)
        .send()?;

    if response.status() == 404 {
        return Err(RemoteAccessError::GameNotFound);
    }
    if response.status() != 200 {
        return Err(RemoteAccessError::InvalidCodeError(
            response.status().into(),
        ));
    }

    let game = response.json::<Game>()?;
    state_handle.games.insert(meta.clone(), game.clone());

    let mut db_handle = DB.borrow_data_mut().unwrap();

    db_handle
        .applications
        .statuses
        .entry(meta.clone())
        .or_insert(ApplicationStatus::Remote {});
    drop(db_handle);

    let status = DownloadStatusManager::fetch_state(&meta);

    let data = FetchGameStruct {
        game: game.clone(),
        status,
    };

    Ok(data)
}

#[tauri::command]
pub fn fetch_game(id: DownloadableMetadata, app: tauri::AppHandle) -> Result<FetchGameStruct, String> {
    let result = fetch_game_logic(id, app);

    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }

    Ok(result.unwrap())
}

#[tauri::command]
pub fn fetch_game_status(meta: Arc<DownloadableMetadata>) -> Result<ApplicationStatusWithTransient, String> {
    let status = DownloadStatusManager::fetch_state(&meta);

    Ok(status)
}

fn fetch_game_verion_options_logic<'a>(
    meta: DownloadableMetadata,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersionOption>, RemoteAccessError> {
    let base_url = DB.fetch_base_url();

    let endpoint =
        base_url.join(format!("/api/v1/client/metadata/versions?id={}", meta.id).as_str())?;
    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(endpoint.to_string())
        .header("Authorization", header)
        .send()?;

    if response.status() != 200 {
        return Err(RemoteAccessError::InvalidCodeError(
            response.status().into(),
        ));
    }

    let data = response.json::<Vec<GameVersionOption>>()?;

    let state_lock = state.lock().unwrap();
    let process_manager_lock = state_lock.process_manager.lock().unwrap();
    let data = data
        .into_iter()
        .filter(|v| process_manager_lock.valid_platform(&v.platform).unwrap())
        .collect::<Vec<GameVersionOption>>();
    drop(process_manager_lock);
    drop(state_lock);

    Ok(data)
}

#[tauri::command]
pub fn fetch_game_verion_options<'a>(
    game_id: DownloadableMetadata,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersionOption>, String> {
    fetch_game_verion_options_logic(game_id, state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_game(
    game_id: Arc<DownloadableMetadata>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let state_lock = state.lock().unwrap();
    state_lock.download_manager.uninstall_application(game_id);
    drop(state_lock);

    Ok(())
}

pub fn push_game_update(app_handle: &AppHandle, meta: DownloadableMetadata, status: ApplicationStatusWithTransient) {
    app_handle
        .emit(
            &format!("update_game/{}", meta.id),
            GameUpdateEvent {
                game_id: meta.id,
                status,
            },
        )
        .unwrap();
}

pub fn on_game_complete(
    meta: Arc<DownloadableMetadata>,
    install_dir: String,
    app_handle: &AppHandle,
) -> Result<(), RemoteAccessError> {
    // Fetch game version information from remote
    let base_url = DB.fetch_base_url();

    let endpoint = base_url.join(
        format!(
            "/api/v1/client/metadata/version?id={}&version={}",
            meta.id,
            encode(&meta.version)
        )
        .as_str(),
    )?;
    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(endpoint.to_string())
        .header("Authorization", header)
        .send()?;

    let data = response.json::<ApplicationVersion>()?;

    let mut handle = DB.borrow_data_mut().unwrap();
    handle
        .applications
        .versions
        .entry(*meta.clone())
        .or_default()
        .insert(meta.version.clone(), data.clone());
    drop(handle);
    DB.save().unwrap();

    let status = if data.setup_command.is_empty() {
        ApplicationStatus::Installed {
            version_name: (*meta.version.clone()).to_string(),
            install_dir,
        }
    } else {
        ApplicationStatus::SetupRequired {
            version_name: (*meta.version.clone()).to_string(),
            install_dir,
        }
    };

    let mut db_handle = DB.borrow_data_mut().unwrap();
    db_handle
        .applications
        .statuses
        .insert(*meta.clone(), status.clone());
    drop(db_handle);
    DB.save().unwrap();
    app_handle
        .emit(
            &format!("update_game/{}", meta.id),
            GameUpdateEvent {
                game_id: (*meta.id.clone()).to_string(),
                status: (Some(status), None),
            },
        )
        .unwrap();

    Ok(())
}
