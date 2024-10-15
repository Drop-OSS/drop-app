use std::{borrow::BorrowMut, sync::Mutex};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::{auth::generate_authorization_header, db::fetch_base_url, AppState};

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    id: String,
    mName: String,
    mShortDescription: String,
    mDescription: String,
    // mDevelopers
    // mPublishers

    mIconId: String,
    mBannerId: String,
    mCoverId: String,
    mImageLibrary: Vec<String>,
}

#[tauri::command]
pub fn fetch_library(app: AppHandle) -> Result<String, String> {
    let base_url = fetch_base_url();
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

    for game in games.iter() {
        handle.games.insert(game.id.clone(), game.clone());
    }

    drop(handle);

    return Ok(json!(games.clone()).to_string());
}

#[tauri::command]
pub fn fetch_game(id: String, app: tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<Mutex<AppState>>();
    let handle = state.lock().unwrap();
    let game = handle.games.get(&id);
    if game.is_some() {
        return Ok(json!(game.unwrap()).to_string());
    }

    return Ok("".to_string());
}
