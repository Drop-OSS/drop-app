use std::fs::remove_dir_all;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Emitter;
use tokio::spawn;
use tokio::sync::Mutex;

use crate::AppState;
use crate::database::db::{borrow_db_checked, borrow_db_mut_checked};
use crate::database::models::data::{
    ApplicationTransientStatus, DownloadableMetadata, GameDownloadStatus, GameVersion,
};
use crate::download_manager::download_manager_frontend::DownloadStatus;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::state::{GameStatusManager, GameStatusWithTransient};
use crate::remote::auth::generate_authorization_header;
use crate::remote::cache::{cache_object, get_cached_object, get_cached_object_db};
use crate::remote::requests::make_request;
use bitcode::{Decode, Encode};

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchGameStruct {
    game: Game,
    status: GameStatusWithTransient,
    version: Option<GameVersion>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: String,
    m_name: String,
    m_short_description: String,
    m_description: String,
    // mDevelopers
    // mPublishers
    m_icon_object_id: String,
    m_banner_object_id: String,
    m_cover_object_id: String,
    m_image_library_object_ids: Vec<String>,
    m_image_carousel_object_ids: Vec<String>,
}
#[derive(serde::Serialize, Clone)]
pub struct GameUpdateEvent {
    pub game_id: String,
    pub status: (
        Option<GameDownloadStatus>,
        Option<ApplicationTransientStatus>,
    ),
    pub version: Option<GameVersion>,
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

pub async fn fetch_library_logic(
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let header = generate_authorization_header().await;

    let client = reqwest::Client::new();
    let response = make_request(&client, &["/api/v1/client/user/library"], &[], async |f| {
        f.header("Authorization", header)
    })
    .await?
    .send()
    .await?;

    if response.status() != 200 {
        let err = response.json().await.unwrap();
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let mut games: Vec<Game> = response.json().await?;

    let mut handle = state.lock().await;

    let mut db_handle = borrow_db_mut_checked().await;

    for game in games.iter() {
        handle.games.insert(game.id.clone(), game.clone());
        if !db_handle.applications.game_statuses.contains_key(&game.id) {
            db_handle
                .applications
                .game_statuses
                .insert(game.id.clone(), GameDownloadStatus::Remote {});
        }
    }

    // Add games that are installed but no longer in library
    for meta in db_handle.applications.installed_game_version.values() {
        if games.iter().any(|e| e.id == meta.id) {
            continue;
        }
        // We should always have a cache of the object
        // Pass db_handle because otherwise we get a gridlock
        let game = get_cached_object_db::<String, Game>(meta.id.clone(), &db_handle).await?;
        games.push(game);
    }

    drop(handle);
    drop(db_handle);
    cache_object("library", &games).await?;

    Ok(games)
}
pub async fn fetch_library_logic_offline(
    _state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let mut games: Vec<Game> = get_cached_object("library").await?;

    let db_handle = borrow_db_checked().await;

    games.retain(|game| {
        db_handle
            .applications
            .installed_game_version
            .contains_key(&game.id)
    });

    Ok(games)
}
pub async fn fetch_game_logic(
    id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let mut state_handle = state.lock().await;

    let handle = borrow_db_checked().await;

    let metadata_option = handle.applications.installed_game_version.get(&id);
    let version = match metadata_option {
        None => None,
        Some(metadata) => Some(
            handle
                .applications
                .game_versions
                .get(&metadata.id)
                .unwrap()
                .get(metadata.version.as_ref().unwrap())
                .unwrap()
                .clone(),
        ),
    };
    drop(handle);

    let game = state_handle.games.get(&id);
    if let Some(game) = game {
        let status = GameStatusManager::fetch_state(&id).await;

        let data = FetchGameStruct {
            game: game.clone(),
            status,
            version,
        };

        cache_object(id, game).await?;

        return Ok(data);
    }
    let client = reqwest::Client::new();
    let response = make_request(&client, &["/api/v1/client/game/", &id], &[], async |r| {
        r.header("Authorization", generate_authorization_header().await)
    })
    .await?
    .send()
    .await?;

    if response.status() == 404 {
        return Err(RemoteAccessError::GameNotFound(id));
    }
    if response.status() != 200 {
        let err = response.json().await.unwrap();
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let game: Game = response.json().await?;
    state_handle.games.insert(id.clone(), game.clone());

    let mut db_handle = borrow_db_mut_checked().await;

    db_handle
        .applications
        .game_statuses
        .entry(id.clone())
        .or_insert(GameDownloadStatus::Remote {});
    drop(db_handle);

    let status = GameStatusManager::fetch_state(&id).await;

    let data = FetchGameStruct {
        game: game.clone(),
        status,
        version,
    };

    cache_object(id, &game).await?;

    Ok(data)
}

pub async fn fetch_game_logic_offline(
    id: String,
    _state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let handle = borrow_db_checked().await;
    let metadata_option = handle.applications.installed_game_version.get(&id);
    let version = match metadata_option {
        None => None,
        Some(metadata) => Some(
            handle
                .applications
                .game_versions
                .get(&metadata.id)
                .unwrap()
                .get(metadata.version.as_ref().unwrap())
                .unwrap()
                .clone(),
        ),
    };
    drop(handle);

    let status = GameStatusManager::fetch_state(&id).await;
    let game = get_cached_object::<String, Game>(id).await?;

    Ok(FetchGameStruct {
        game,
        status,
        version,
    })
}

pub async fn fetch_game_verion_options_logic(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    let client = reqwest::Client::new();

    let response = make_request(
        &client,
        &["/api/v1/client/game/versions"],
        &[("id", &game_id)],
        async |r| r.header("Authorization", generate_authorization_header().await),
    )
    .await?
    .send()
    .await?;

    if response.status() != 200 {
        let err = response.json().await.unwrap();
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let data: Vec<GameVersion> = response.json().await?;

    let state_lock = state.lock().await;
    let process_manager_lock = state_lock.process_manager.lock().await;
    let data: Vec<GameVersion> = data
        .into_iter()
        .filter(|v| process_manager_lock.valid_platform(&v.platform).unwrap())
        .collect();
    drop(process_manager_lock);
    drop(state_lock);

    Ok(data)
}

pub async fn uninstall_game_logic(meta: DownloadableMetadata, app_handle: &AppHandle) {
    debug!("triggered uninstall for agent");
    let mut db_handle = borrow_db_mut_checked().await;
    db_handle
        .applications
        .transient_statuses
        .entry(meta.clone())
        .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});

    push_game_update(
        app_handle,
        &meta.id,
        None,
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
        GameDownloadStatus::PartiallyInstalled {
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
        spawn(async move {
            match remove_dir_all(install_dir) {
                Err(e) => {
                    error!("{e}");
                }
                Ok(_) => {
                    let mut db_handle = borrow_db_mut_checked().await;
                    db_handle.applications.transient_statuses.remove(&meta);
                    db_handle
                        .applications
                        .installed_game_version
                        .remove(&meta.id);
                    db_handle
                        .applications
                        .game_statuses
                        .entry(meta.id.clone())
                        .and_modify(|e| *e = GameDownloadStatus::Remote {});
                    drop(db_handle);

                    debug!("uninstalled game id {}", &meta.id);
                    app_handle.emit("update_library", ()).unwrap();

                    push_game_update(
                        &app_handle,
                        &meta.id,
                        None,
                        (Some(GameDownloadStatus::Remote {}), None),
                    );
                }
            }
        });
    } else {
        warn!("invalid previous state for uninstall, failing silently.")
    }
}

pub async fn get_current_meta(game_id: &String) -> Option<DownloadableMetadata> {
    borrow_db_checked()
        .await
        .applications
        .installed_game_version
        .get(game_id)
        .cloned()
}

pub async fn on_game_incomplete(
    meta: &DownloadableMetadata,
    install_dir: String,
    app_handle: &AppHandle,
) -> Result<(), RemoteAccessError> {
    // Fetch game version information from remote
    if meta.version.is_none() {
        return Err(RemoteAccessError::GameNotFound(meta.id.clone()));
    }

    let client = reqwest::Client::new();
    let response = make_request(
        &client,
        &["/api/v1/client/game/version"],
        &[
            ("id", &meta.id),
            ("version", meta.version.as_ref().unwrap()),
        ],
        async |f| f.header("Authorization", generate_authorization_header().await),
    )
    .await?
    .send()
    .await?;

    let game_version: GameVersion = response.json().await?;

    let mut handle = borrow_db_mut_checked().await;
    handle
        .applications
        .game_versions
        .entry(meta.id.clone())
        .or_default()
        .insert(meta.version.clone().unwrap(), game_version.clone());
    handle
        .applications
        .installed_game_version
        .insert(meta.id.clone(), meta.clone());

    let status = GameDownloadStatus::PartiallyInstalled {
        version_name: meta.version.clone().unwrap(),
        install_dir,
    };

    handle
        .applications
        .game_statuses
        .insert(meta.id.clone(), status.clone());
    drop(handle);
    app_handle
        .emit(
            &format!("update_game/{}", meta.id),
            GameUpdateEvent {
                game_id: meta.id.clone(),
                status: (Some(status), None),
                version: Some(game_version),
            },
        )
        .unwrap();

    Ok(())
}

pub async fn on_game_complete(
    meta: &DownloadableMetadata,
    install_dir: String,
    app_handle: &AppHandle,
) -> Result<(), RemoteAccessError> {
    // Fetch game version information from remote
    if meta.version.is_none() {
        return Err(RemoteAccessError::GameNotFound(meta.id.clone()));
    }

    let header = generate_authorization_header().await;

    let client = reqwest::Client::new();
    let response = make_request(
        &client,
        &["/api/v1/client/game/version"],
        &[
            ("id", &meta.id),
            ("version", meta.version.as_ref().unwrap()),
        ],
        async |f| f.header("Authorization", header),
    )
    .await?
    .send()
    .await?;

    let game_version: GameVersion = response.json().await?;

    let mut handle = borrow_db_mut_checked().await;
    handle
        .applications
        .game_versions
        .entry(meta.id.clone())
        .or_default()
        .insert(meta.version.clone().unwrap(), game_version.clone());
    handle
        .applications
        .installed_game_version
        .insert(meta.id.clone(), meta.clone());

    drop(handle);

    let status = if game_version.setup_command.is_empty() {
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

    let mut db_handle = borrow_db_mut_checked().await;
    db_handle
        .applications
        .game_statuses
        .insert(meta.id.clone(), status.clone());
    drop(db_handle);
    app_handle
        .emit(
            &format!("update_game/{}", meta.id),
            GameUpdateEvent {
                game_id: meta.id.clone(),
                status: (Some(status), None),
                version: Some(game_version),
            },
        )
        .unwrap();

    Ok(())
}

pub fn push_game_update(
    app_handle: &AppHandle,
    game_id: &String,
    version: Option<GameVersion>,
    status: GameStatusWithTransient,
) {
    app_handle
        .emit(
            &format!("update_game/{game_id}"),
            GameUpdateEvent {
                game_id: game_id.clone(),
                status,
                version,
            },
        )
        .unwrap();
}
