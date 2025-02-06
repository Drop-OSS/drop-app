use std::fs::remove_dir_all;
use std::sync::Mutex;
use std::thread::spawn;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::{AppHandle, Manager};

use crate::database::db::{borrow_db_checked, borrow_db_mut_checked, save_db, GameVersion};
use crate::database::db::{ApplicationTransientStatus, GameDownloadStatus};
use crate::download_manager::download_manager::DownloadStatus;
use crate::download_manager::downloadable_metadata::DownloadableMetadata;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::state::{GameStatusManager, GameStatusWithTransient};
use crate::remote::auth::generate_authorization_header;
use crate::remote::cache::{cache_object, get_cached_object};
use crate::remote::requests::make_request;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct FetchGameStruct {
    game: Game,
    status: GameStatusWithTransient,
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
    pub status: (
        Option<GameDownloadStatus>,
        Option<ApplicationTransientStatus>,
    ),
}

#[derive(Serialize, Clone)]
pub struct QueueUpdateEventQueueData {
    pub meta: DownloadableMetadata,
    pub status: DownloadStatus,
    pub progress: f64,
    pub current: usize,
    pub max: usize,
}

#[derive(serde::Serialize, Clone)]
pub struct QueueUpdateEvent {
    pub queue: Vec<QueueUpdateEventQueueData>,
}

#[derive(serde::Serialize, Clone)]
pub struct StatsUpdateEvent {
    pub speed: usize,
    pub time: usize,
}

pub fn fetch_library_logic(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    println!("Making library request");
    let response = make_request(&client, &["/api/v1/client/user/library"], &[], |f| {
        f.header("Authorization", header)
    })?
    .send()?;

    if response.status() != 200 {
        let err = response.json().unwrap();
        warn!("{:?}", err);
        return Err(RemoteAccessError::InvalidResponse(err));
    }
    println!("Getting Games");

    let games: Vec<Game> = response.json()?;

    let mut handle = state.lock().unwrap();

    let mut db_handle = borrow_db_mut_checked();

    for game in games.iter() {
        handle.games.insert(game.id.clone(), game.clone());
        if !db_handle.applications.game_statuses.contains_key(&game.id) {
            db_handle
                .applications
                .game_statuses
                .insert(game.id.clone(), GameDownloadStatus::Remote {});
        }
    }
    drop(handle);
    drop(db_handle);
    println!("Caching");
    cache_object("library", &games)?;

    println!("Finished caching");

    Ok(games)
}
pub fn fetch_library_logic_offline(
    _state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let mut games: Vec<Game> = get_cached_object("library")?;

    let db_handle = borrow_db_checked();

    games.retain(|game| {
        db_handle
            .applications
            .installed_game_version
            .contains_key(&game.id)
    });

    Ok(games)
}
pub fn fetch_game_logic(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let mut state_handle = state.lock().unwrap();

    let game = state_handle.games.get(&id);
    if let Some(game) = game {
        let status = GameStatusManager::fetch_state(&id);

        let data = FetchGameStruct {
            game: game.clone(),
            status,
        };

        return Ok(data);
    }
    let client = reqwest::blocking::Client::new();
    let response = make_request(&client, &["/api/v1/game/", &id], &[], |r| {
        r.header("Authorization", generate_authorization_header())
    })?
    .send()?;

    if response.status() == 404 {
        return Err(RemoteAccessError::GameNotFound);
    }
    if response.status() != 200 {
        let err = response.json().unwrap();
        warn!("{:?}", err);
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let game: Game = response.json()?;
    state_handle.games.insert(id.clone(), game.clone());

    let mut db_handle = borrow_db_mut_checked();

    db_handle
        .applications
        .game_statuses
        .entry(id.clone())
        .or_insert(GameDownloadStatus::Remote {});
    drop(db_handle);

    let status = GameStatusManager::fetch_state(&id);

    let data = FetchGameStruct {
        game: game.clone(),
        status,
    };

    cache_object(id, &data)?;

    Ok(data)
}

pub fn fetch_game_logic_offline(
    id: String,
    _state: tauri::State<'_, Mutex<AppState>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    get_cached_object(id)
}

pub fn fetch_game_verion_options_logic(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    let client = reqwest::blocking::Client::new();

    let response = make_request(
        &client,
        &["/api/v1/client/game/versions"],
        &[("id", &game_id)],
        |r| r.header("Authorization", generate_authorization_header()),
    )?
    .send()?;

    if response.status() != 200 {
        let err = response.json().unwrap();
        warn!("{:?}", err);
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let data: Vec<GameVersion> = response.json()?;

    let state_lock = state.lock().unwrap();
    let process_manager_lock = state_lock.process_manager.lock().unwrap();
    let data: Vec<GameVersion> = data
        .into_iter()
        .filter(|v| process_manager_lock.valid_platform(&v.platform).unwrap())
        .collect();
    drop(process_manager_lock);
    drop(state_lock);

    Ok(data)
}

pub fn uninstall_game_logic(meta: DownloadableMetadata, app_handle: &AppHandle) {
    println!("triggered uninstall for agent");
    let mut db_handle = borrow_db_mut_checked();
    db_handle
        .applications
        .transient_statuses
        .entry(meta.clone())
        .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});

    push_game_update(
        app_handle,
        &meta.id,
        (None, Some(ApplicationTransientStatus::Uninstalling {})),
    );

    let previous_state = db_handle.applications.game_statuses.get(&meta.id).cloned();
    if previous_state.is_none() {
        warn!("uninstall job doesn't have previous state, failing silently");
        return;
    }
    let previous_state = previous_state.unwrap();
    if let Some((_, install_dir)) = match previous_state {
        GameDownloadStatus::Installed {
            version_name,
            install_dir,
        } => Some((version_name, install_dir)),
        GameDownloadStatus::SetupRequired {
            version_name,
            install_dir,
        } => Some((version_name, install_dir)),
        _ => None,
    } {
        db_handle
            .applications
            .transient_statuses
            .entry(meta.clone())
            .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});
        drop(db_handle);

        let app_handle = app_handle.clone();
        spawn(move || match remove_dir_all(install_dir) {
            Err(e) => {
                error!("{}", e);
            }
            Ok(_) => {
                let mut db_handle = borrow_db_mut_checked();
                db_handle.applications.transient_statuses.remove(&meta);
                db_handle
                    .applications
                    .game_statuses
                    .entry(meta.id.clone())
                    .and_modify(|e| *e = GameDownloadStatus::Remote {});
                drop(db_handle);
                save_db();

                debug!("uninstalled game id {}", &meta.id);

                push_game_update(
                    &app_handle,
                    &meta.id,
                    (Some(GameDownloadStatus::Remote {}), None),
                );
            }
        });
    }
}

pub fn get_current_meta(game_id: &String) -> Option<DownloadableMetadata> {
    borrow_db_checked()
        .applications
        .installed_game_version
        .get(game_id)
        .cloned()
}

pub fn on_game_complete(
    meta: &DownloadableMetadata,
    install_dir: String,
    app_handle: &AppHandle,
) -> Result<(), RemoteAccessError> {
    // Fetch game version information from remote
    if meta.version.is_none() {
        return Err(RemoteAccessError::GameNotFound);
    }

    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = make_request(
        &client,
        &["/api/v1/client/metadata/version"],
        &[
            ("id", &meta.id),
            ("version", meta.version.as_ref().unwrap()),
        ],
        |f| f.header("Authorization", header),
    )?
    .send()?;

    let data: GameVersion = response.json()?;

    let mut handle = borrow_db_mut_checked();
    handle
        .applications
        .game_versions
        .entry(meta.id.clone())
        .or_default()
        .insert(meta.version.clone().unwrap(), data.clone());
    handle
        .applications
        .installed_game_version
        .insert(meta.id.clone(), meta.clone());

    drop(handle);
    save_db();

    let status = if data.setup_command.is_empty() {
        GameDownloadStatus::Installed {
            version_name: meta.version.clone().unwrap(),
            install_dir,
        }
    } else {
        GameDownloadStatus::SetupRequired {
            version_name: meta.version.clone().unwrap(),
            install_dir,
        }
    };

    let mut db_handle = borrow_db_mut_checked();
    db_handle
        .applications
        .game_statuses
        .insert(meta.id.clone(), status.clone());
    drop(db_handle);
    save_db();
    app_handle
        .emit(
            &format!("update_game/{}", meta.id),
            GameUpdateEvent {
                game_id: meta.id.clone(),
                status: (Some(status), None),
            },
        )
        .unwrap();

    Ok(())
}

pub fn push_game_update(app_handle: &AppHandle, game_id: &String, status: GameStatusWithTransient) {
    app_handle
        .emit(
            &format!("update_game/{}", game_id),
            GameUpdateEvent {
                game_id: game_id.clone(),
                status,
            },
        )
        .unwrap();
}
