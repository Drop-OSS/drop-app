use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::{auth::generate_authorization_header, AppState, DB};
use crate::db::DatabaseImpls;
use crate::db::DatabaseGameStatus;

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

#[tauri::command]
pub fn fetch_library(app: AppHandle) -> Result<String, String> {
    let base_url = DB.fetch_base_url();
    let library_url = base_url.join("/api/v1/client/user/library").unwrap();

    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(library_url.to_string())
        .header("Authorization", header)
        .send()
        .unwrap();

    if response.status() != 200 {
        return Err(format!(
            "Library fetch request failed with {}",
            response.status()
        ));
    }

    // Keep as string
    let games = response.json::<Vec<Game>>().unwrap();

    let state = app.state::<Mutex<AppState>>();
    let mut handle = state.lock().unwrap();

    let mut db_handle = DB.borrow_data_mut().unwrap();

    for game in games.iter() {
        handle.games.insert(game.id.clone(), game.clone());
        if !db_handle.games.games_statuses.contains_key(&game.id) {
            db_handle
                .games
                .games_statuses
                .insert(game.id.clone(), DatabaseGameStatus::Remote);
        }
    }

    drop(handle);

    Ok(json!(games.clone()).to_string())
}

#[tauri::command]
pub fn fetch_game(id: String, app: tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<Mutex<AppState>>();
    let handle = state.lock().unwrap();
    let game = handle.games.get(&id);
    if let Some(game) = game {
        let db_handle = DB.borrow_data().unwrap();

        let data = FetchGameStruct {
            game: game.clone(),
            status: db_handle.games.games_statuses.get(&game.id).unwrap().clone(),
        };

        return Ok(json!(data).to_string());
    }

    Err("".to_string())
}
