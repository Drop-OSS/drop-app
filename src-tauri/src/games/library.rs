use std::fs::remove_dir_all;
use std::sync::Mutex;
use std::thread::spawn;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Emitter;

use crate::AppState;
use crate::database::db::{borrow_db_checked, borrow_db_mut_checked};
use crate::database::models::data::Database;
use crate::database::models::data::{
    ApplicationTransientStatus, DownloadableMetadata, GameDownloadStatus, GameVersion,
};
use crate::download_manager::download_manager_frontend::DownloadStatus;
use crate::error::drop_server_error::DropServerError;
use crate::error::library_error::LibraryError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::state::{GameStatusManager, GameStatusWithTransient};
use crate::remote::auth::generate_authorization_header;
use crate::remote::cache::cache_object_db;
use crate::remote::cache::{cache_object, get_cached_object, get_cached_object_db};
use crate::remote::requests::generate_url;
use crate::remote::utils::DROP_CLIENT_ASYNC;
use crate::remote::utils::DROP_CLIENT_SYNC;
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
    let client = DROP_CLIENT_ASYNC.clone();
    let response = generate_url(&["/api/v1/client/user/library"], &[])?;
    let response = client
        .get(response)
        .header("Authorization", generate_authorization_header())
        .send()
        .await?;

    if response.status() != 200 {
        let err = response.json().await.unwrap_or(DropServerError {
            status_code: 500,
            status_message: "Invalid response from server.".to_owned(),
        });
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let mut games: Vec<Game> = response.json().await?;

    let mut handle = state.lock().unwrap();

    let mut db_handle = borrow_db_mut_checked();

    for game in &games {
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
        let game = match get_cached_object_db::<Game>(&meta.id.clone(), &db_handle) {
            Ok(game) => game,
            Err(err) => {
                warn!(
                    "{} is installed, but encountered error fetching its error: {}.",
                    meta.id, err
                );
                continue;
            }
        };
        games.push(game);
    }

    drop(handle);
    drop(db_handle);
    cache_object("library", &games)?;

    Ok(games)
}
pub async fn fetch_library_logic_offline(
    _state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let mut games: Vec<Game> = get_cached_object("library")?;

    let db_handle = borrow_db_checked();

    games.retain(|game| {
        matches!(
            &db_handle
                .applications
                .game_statuses
                .get(&game.id)
                .unwrap_or(&GameDownloadStatus::Remote {}),
            GameDownloadStatus::Installed { .. } | GameDownloadStatus::SetupRequired { .. }
        )
    });

    Ok(games)
}
pub async fn fetch_game_logic(
    id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let version = {
        let state_handle = state.lock().unwrap();

        let db_lock = borrow_db_checked();

        let metadata_option = db_lock.applications.installed_game_version.get(&id);
        let version = match metadata_option {
            None => None,
            Some(metadata) => db_lock
                .applications
                .game_versions
                .get(&metadata.id)
                .map(|v| v.get(metadata.version.as_ref().unwrap()).unwrap())
                .cloned(),
        };

        let game = state_handle.games.get(&id);
        if let Some(game) = game {
            let status = GameStatusManager::fetch_state(&id, &db_lock);

            let data = FetchGameStruct {
                game: game.clone(),
                status,
                version,
            };

            cache_object_db(&id, game, &db_lock)?;

            return Ok(data);
        }

        version
    };

    let client = DROP_CLIENT_ASYNC.clone();
    let response = generate_url(&["/api/v1/client/game/", &id], &[])?;
    let response = client
        .get(response)
        .header("Authorization", generate_authorization_header())
        .send()
        .await?;

    if response.status() == 404 {
        let offline_fetch = fetch_game_logic_offline(id.clone(), state).await;
        if let Ok(fetch_data) = offline_fetch {
            return Ok(fetch_data);
        }

        return Err(RemoteAccessError::GameNotFound(id));
    }
    if response.status() != 200 {
        let err = response.json().await.unwrap();
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let game: Game = response.json().await?;

    let mut state_handle = state.lock().unwrap();
    state_handle.games.insert(id.clone(), game.clone());

    let mut db_handle = borrow_db_mut_checked();

    db_handle
        .applications
        .game_statuses
        .entry(id.clone())
        .or_insert(GameDownloadStatus::Remote {});

    let status = GameStatusManager::fetch_state(&id, &db_handle);

    drop(db_handle);

    let data = FetchGameStruct {
        game: game.clone(),
        status,
        version,
    };

    cache_object(&id, &game)?;

    Ok(data)
}

pub async fn fetch_game_logic_offline(
    id: String,
    _state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let db_handle = borrow_db_checked();
    let metadata_option = db_handle.applications.installed_game_version.get(&id);
    let version = match metadata_option {
        None => None,
        Some(metadata) => db_handle
            .applications
            .game_versions
            .get(&metadata.id)
            .map(|v| v.get(metadata.version.as_ref().unwrap()).unwrap())
            .cloned(),
    };

    let status = GameStatusManager::fetch_state(&id, &db_handle);
    let game = get_cached_object::<Game>(&id)?;

    drop(db_handle);

    Ok(FetchGameStruct {
        game,
        status,
        version,
    })
}

pub async fn fetch_game_version_options_logic(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    let client = DROP_CLIENT_ASYNC.clone();

    let response = generate_url(&["/api/v1/client/game/versions"], &[("id", &game_id)])?;
    let response = client
        .get(response)
        .header("Authorization", generate_authorization_header())
        .send()
        .await?;

    if response.status() != 200 {
        let err = response.json().await.unwrap();
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let data: Vec<GameVersion> = response.json().await?;

    let state_lock = state.lock().unwrap();
    let process_manager_lock = state_lock.process_manager.lock().unwrap();
    let data: Vec<GameVersion> = data
        .into_iter()
        .filter(|v| {
            process_manager_lock
                .valid_platform(&v.platform, &state_lock)
                .unwrap()
        })
        .collect();
    drop(process_manager_lock);
    drop(state_lock);

    Ok(data)
}

/**
 * Called by:
 *  - on_cancel, when cancelled, for obvious reasons
 *  - when downloading, so if drop unexpectedly quits, we can resume the download. hidden by the "Downloading..." transient state, though
 *  - when scanning, to import the game
 */
pub fn set_partially_installed(
    meta: &DownloadableMetadata,
    install_dir: String,
    app_handle: Option<&AppHandle>,
) {
    set_partially_installed_db(&mut borrow_db_mut_checked(), meta, install_dir, app_handle);
}

pub fn set_partially_installed_db(
    db_lock: &mut Database,
    meta: &DownloadableMetadata,
    install_dir: String,
    app_handle: Option<&AppHandle>,
) {
    db_lock.applications.transient_statuses.remove(meta);
    db_lock.applications.game_statuses.insert(
        meta.id.clone(),
        GameDownloadStatus::PartiallyInstalled {
            version_name: meta.version.as_ref().unwrap().clone(),
            install_dir,
        },
    );
    db_lock
        .applications
        .installed_game_version
        .insert(meta.id.clone(), meta.clone());

    if let Some(app_handle) = app_handle {
        push_game_update(
            app_handle,
            &meta.id,
            None,
            GameStatusManager::fetch_state(&meta.id, db_lock),
        );
    }
}

pub fn uninstall_game_logic(meta: DownloadableMetadata, app_handle: &AppHandle) {
    debug!("triggered uninstall for agent");
    let mut db_handle = borrow_db_mut_checked();
    db_handle
        .applications
        .transient_statuses
        .entry(meta.clone())
        .and_modify(|v| *v = ApplicationTransientStatus::Uninstalling {});

    push_game_update(
        app_handle,
        &meta.id,
        None,
        GameStatusManager::fetch_state(&meta.id, &db_handle),
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
        spawn(move || {
            if let Err(e) = remove_dir_all(install_dir) {
                error!("{e}");
            } else {
                let mut db_handle = borrow_db_mut_checked();
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
                let _ = db_handle.applications.transient_statuses.remove(&meta);

                push_game_update(
                    &app_handle,
                    &meta.id,
                    None,
                    GameStatusManager::fetch_state(&meta.id, &db_handle),
                );

                debug!("uninstalled game id {}", &meta.id);
                app_handle.emit("update_library", ()).unwrap();

                drop(db_handle);
            }
        });
    } else {
        warn!("invalid previous state for uninstall, failing silently.");
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
        return Err(RemoteAccessError::GameNotFound(meta.id.clone()));
    }

    let client = DROP_CLIENT_SYNC.clone();
    let response = generate_url(
        &["/api/v1/client/game/version"],
        &[
            ("id", &meta.id),
            ("version", meta.version.as_ref().unwrap()),
        ],
    )?;
    let response = client
        .get(response)
        .header("Authorization", generate_authorization_header())
        .send()?;

    let game_version: GameVersion = response.json()?;

    let mut handle = borrow_db_mut_checked();
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

    let mut db_handle = borrow_db_mut_checked();
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendGameOptions {
    launch_string: String,
}

#[tauri::command]
pub fn update_game_configuration(
    game_id: String,
    options: FrontendGameOptions,
) -> Result<(), LibraryError> {
    let mut handle = borrow_db_mut_checked();
    let installed_version = handle
        .applications
        .installed_game_version
        .get(&game_id)
        .ok_or(LibraryError::MetaNotFound(game_id))?;

    let id = installed_version.id.clone();
    let version = installed_version.version.clone().unwrap();

    let mut existing_configuration = handle
        .applications
        .game_versions
        .get(&id)
        .unwrap()
        .get(&version)
        .unwrap()
        .clone();

    // Add more options in here
    existing_configuration.launch_command_template = options.launch_string;

    // Add no more options past here

    handle
        .applications
        .game_versions
        .get_mut(&id)
        .unwrap()
        .insert(version.to_string(), existing_configuration);

    Ok(())
}
