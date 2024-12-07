use std::collections::{HashMap, VecDeque};
use std::fmt::format;
use std::sync::Mutex;

use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use urlencoding::encode;

use crate::db::DatabaseGameStatus;
use crate::db::DatabaseImpls;
use crate::db::{self, GameVersion};
use crate::downloads::download_manager::GameDownloadStatus;
use crate::remote::RemoteAccessError;
use crate::{auth::generate_authorization_header, AppState, DB};

#[derive(serde::Serialize)]
struct FetchGameStruct {
    game: Game,
    status: DatabaseGameStatus,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: String,
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
    pub status: DatabaseGameStatus,
}

#[derive(Serialize, Clone)]
pub struct QueueUpdateEventQueueData {
    pub id: String,
    pub status: GameDownloadStatus,
    pub progress: f64,
}

#[derive(serde::Serialize, Clone)]
pub struct QueueUpdateEvent {
    pub queue: Vec<QueueUpdateEventQueueData>,
}

// Game version with some fields missing and size information
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersionOption {
    version_index: usize,
    version_name: String,
    platform: String,
    setup_command: String,
    launch_command: String,
    delta: bool,
    // total_size: usize,
}

fn fetch_library_logic(app: AppHandle) -> Result<String, RemoteAccessError> {
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
        handle.games.insert(game.id.clone(), game.clone());
        if !db_handle.games.games_statuses.contains_key(&game.id) {
            db_handle
                .games
                .games_statuses
                .insert(game.id.clone(), DatabaseGameStatus::Remote {});
        }
    }

    drop(handle);

    Ok(json!(games.clone()).to_string())
}

#[tauri::command]
pub fn fetch_library(app: AppHandle) -> Result<String, String> {
    fetch_library_logic(app).map_err(|e| e.to_string())
}

fn fetch_game_logic(id: String, app: tauri::AppHandle) -> Result<String, RemoteAccessError> {
    let state = app.state::<Mutex<AppState>>();
    let mut state_handle = state.lock().unwrap();

    let game = state_handle.games.get(&id);
    if let Some(game) = game {
        let db_handle = DB.borrow_data().unwrap();

        let data = FetchGameStruct {
            game: game.clone(),
            status: db_handle
                .games
                .games_statuses
                .get(&game.id)
                .unwrap()
                .clone(),
        };

        return Ok(json!(data).to_string());
    }

    let base_url = DB.fetch_base_url();

    let endpoint = base_url.join(&format!("/api/v1/game/{}", id))?;
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
    state_handle.games.insert(id.clone(), game.clone());

    let mut db_handle = DB.borrow_data_mut().unwrap();

    if !db_handle.games.games_statuses.contains_key(&id) {
        db_handle
            .games
            .games_statuses
            .insert(id, DatabaseGameStatus::Remote {});
    }

    let data = FetchGameStruct {
        game: game.clone(),
        status: db_handle
            .games
            .games_statuses
            .get(&game.id)
            .unwrap()
            .clone(),
    };

    return Ok(json!(data).to_string());
}

#[tauri::command]
pub fn fetch_game(id: String, app: tauri::AppHandle) -> Result<String, String> {
    let result = fetch_game_logic(id, app);

    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }

    Ok(result.unwrap())
}

#[tauri::command]
pub fn fetch_game_status(id: String) -> Result<DatabaseGameStatus, String> {
    let db_handle = DB.borrow_data().unwrap();
    let status = db_handle
        .games
        .games_statuses
        .get(&id)
        .unwrap_or(&DatabaseGameStatus::Remote {})
        .clone();
    drop(db_handle);

    return Ok(status);
}

fn fetch_game_verion_options_logic(
    game_id: String,
) -> Result<Vec<GameVersionOption>, RemoteAccessError> {
    let base_url = DB.fetch_base_url();

    let endpoint =
        base_url.join(format!("/api/v1/client/metadata/versions?id={}", game_id).as_str())?;
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

    return Ok(data);
}

#[tauri::command]
pub fn fetch_game_verion_options(game_id: String) -> Result<Vec<GameVersionOption>, String> {
    fetch_game_verion_options_logic(game_id).map_err(|e| e.to_string())
}

pub fn on_game_complete(
    game_id: String,
    version_name: String,
    app_handle: &AppHandle,
) -> Result<(), RemoteAccessError> {
    // Fetch game version information from remote
    let base_url = DB.fetch_base_url();

    let endpoint = base_url.join(
        format!(
            "/api/v1/client/metadata/version?id={}&version={}",
            game_id,
            encode(&version_name)
        )
        .as_str(),
    )?;
    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(endpoint.to_string())
        .header("Authorization", header)
        .send()?;

    let data = response.json::<GameVersion>()?;

    let mut handle = DB.borrow_data_mut().unwrap();
    handle
        .games
        .game_versions
        .entry(game_id.clone())
        .or_insert(HashMap::new())
        .insert(version_name.clone(), data.clone());
    drop(handle);
    DB.save().unwrap();

    let status = if data.setup_command.is_empty() {
        DatabaseGameStatus::Installed { version_name }
    } else {
        DatabaseGameStatus::SetupRequired { version_name }
    };

    let mut db_handle = DB.borrow_data_mut().unwrap();
    db_handle
        .games
        .games_statuses
        .insert(game_id.clone(), status.clone());
    drop(db_handle);
    DB.save().unwrap();
    app_handle
        .emit(
            &format!("update_game/{}", game_id),
            GameUpdateEvent {
                game_id: game_id,
                status: status,
            },
        )
        .unwrap();

    Ok(())
}
