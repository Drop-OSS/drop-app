use std::sync::nonpoison::Mutex;

use database::{GameDownloadStatus, GameVersion, borrow_db_checked, borrow_db_mut_checked};
use games::{
    downloads::error::LibraryError,
    library::{FetchGameStruct, FrontendGameOptions, Game, get_current_meta, uninstall_game_logic},
    state::{GameStatusManager, GameStatusWithTransient},
};
use log::warn;
use process::PROCESS_MANAGER;
use remote::{
    auth::generate_authorization_header,
    cache::{cache_object, cache_object_db, get_cached_object, get_cached_object_db},
    error::{DropServerError, RemoteAccessError},
    offline,
    requests::generate_url,
    utils::DROP_CLIENT_ASYNC,
};
use tauri::AppHandle;

use crate::AppState;

#[tauri::command]
pub async fn fetch_library(
    state: tauri::State<'_, Mutex<AppState>>,
    hard_refresh: Option<bool>,
) -> Result<Vec<Game>, RemoteAccessError> {
    offline!(
        state,
        fetch_library_logic,
        fetch_library_logic_offline,
        state,
        hard_refresh
    )
    .await
}

pub async fn fetch_library_logic(
    state: tauri::State<'_, Mutex<AppState>>,
    hard_fresh: Option<bool>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let do_hard_refresh = hard_fresh.unwrap_or(false);
    if !do_hard_refresh && let Ok(library) = get_cached_object("library") {
        return Ok(library);
    }

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

    let mut handle = state.lock();

    let mut db_handle = borrow_db_mut_checked();

    for game in &games {
        handle.games.insert(game.id().clone(), game.clone());
        if !db_handle.applications.game_statuses.contains_key(game.id()) {
            db_handle
                .applications
                .game_statuses
                .insert(game.id().clone(), GameDownloadStatus::Remote {});
        }
    }

    // Add games that are installed but no longer in library
    for meta in db_handle.applications.installed_game_version.values() {
        if games.iter().any(|e| *e.id() == meta.id) {
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
    _state: tauri::State<'_, Mutex<AppState>>,
    _hard_refresh: Option<bool>,
) -> Result<Vec<Game>, RemoteAccessError> {
    let mut games: Vec<Game> = get_cached_object("library")?;

    let db_handle = borrow_db_checked();

    games.retain(|game| {
        matches!(
            &db_handle
                .applications
                .game_statuses
                .get(game.id())
                .unwrap_or(&GameDownloadStatus::Remote {}),
            GameDownloadStatus::Installed { .. } | GameDownloadStatus::SetupRequired { .. }
        )
    });

    Ok(games)
}
pub async fn fetch_game_logic(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    let version = {
        let state_handle = state.lock();

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

            let data = FetchGameStruct::new(game.clone(), status, version);

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
        let err = response.json().await?;
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let game: Game = response.json().await?;

    let mut state_handle = state.lock();
    state_handle.games.insert(id.clone(), game.clone());

    let mut db_handle = borrow_db_mut_checked();

    db_handle
        .applications
        .game_statuses
        .entry(id.clone())
        .or_insert(GameDownloadStatus::Remote {});

    let status = GameStatusManager::fetch_state(&id, &db_handle);

    drop(db_handle);

    let data = FetchGameStruct::new(game.clone(), status, version);

    cache_object(&id, &game)?;

    Ok(data)
}

pub async fn fetch_game_version_options_logic(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    let client = DROP_CLIENT_ASYNC.clone();

    let response = generate_url(&["/api/v1/client/game/versions"], &[("id", &game_id)])?;
    let response = client
        .get(response)
        .header("Authorization", generate_authorization_header())
        .send()
        .await?;

    if response.status() != 200 {
        let err = response.json().await?;
        warn!("{err:?}");
        return Err(RemoteAccessError::InvalidResponse(err));
    }

    let data: Vec<GameVersion> = response.json().await?;

    let state_lock = state.lock();
    let process_manager_lock = PROCESS_MANAGER.lock();
    let data: Vec<GameVersion> = data
        .into_iter()
        .filter(|v| process_manager_lock.valid_platform(&v.platform))
        .collect();
    drop(process_manager_lock);
    drop(state_lock);

    Ok(data)
}

pub async fn fetch_game_logic_offline(
    id: String,
    _state: tauri::State<'_, Mutex<AppState>>,
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

    Ok(FetchGameStruct::new(game, status, version))
}

#[tauri::command]
pub async fn fetch_game(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<FetchGameStruct, RemoteAccessError> {
    offline!(
        state,
        fetch_game_logic,
        fetch_game_logic_offline,
        game_id,
        state
    )
    .await
}

#[tauri::command]
pub fn fetch_game_status(id: String) -> GameStatusWithTransient {
    let db_handle = borrow_db_checked();
    GameStatusManager::fetch_state(&id, &db_handle)
}

#[tauri::command]
pub fn uninstall_game(game_id: String, app_handle: AppHandle) -> Result<(), LibraryError> {
    let meta = match get_current_meta(&game_id) {
        Some(data) => data,
        None => return Err(LibraryError::MetaNotFound(game_id)),
    };
    uninstall_game_logic(meta, &app_handle);

    Ok(())
}

#[tauri::command]
pub async fn fetch_game_version_options(
    game_id: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<GameVersion>, RemoteAccessError> {
    fetch_game_version_options_logic(game_id, state).await
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
    let version = installed_version
        .version
        .clone()
        .ok_or(LibraryError::VersionNotFound(id.clone()))?;

    let mut existing_configuration = handle
        .applications
        .game_versions
        .get(&id)
        .unwrap()
        .get(&version)
        .unwrap()
        .clone();

    // Add more options in here
    existing_configuration.launch_command_template = options.launch_string().clone();

    // Add no more options past here

    handle
        .applications
        .game_versions
        .get_mut(&id)
        .unwrap()
        .insert(version.to_string(), existing_configuration);

    Ok(())
}
