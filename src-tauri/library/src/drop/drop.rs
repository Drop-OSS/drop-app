use std::sync::nonpoison::Mutex;

use async_trait::async_trait;
use client::app_state::AppState;
use database::{
    DatabaseAuth, GameDownloadStatus, borrow_db_mut_checked,
    models::{Collection, Game, LibraryProviderMetadata},
};
use log::warn;
use remote::{
    auth::generate_authorization_header,
    cache::{cache_object, get_cached_object, get_cached_object_db},
    error::{DropServerError, RemoteAccessError},
    requests::generate_url,
    utils::DROP_CLIENT_ASYNC,
};
use url::Url;

use crate::{error::LibraryError, provider::LibraryProvider};

pub struct DropLibraryProvider {
    metadata: LibraryProviderMetadata,
    auth: DatabaseAuth,
    base_url: Url,
}

impl DropLibraryProvider {
    pub fn new(metadata: LibraryProviderMetadata, auth: DatabaseAuth, base_url: Url) -> Self {
        Self {
            metadata,
            auth,
            base_url,
        }
    }
}
#[async_trait]
impl LibraryProvider for DropLibraryProvider {
    async fn get_library(
        &self,
        state: &tauri::State<'_, Mutex<AppState>>,
    ) -> Result<Vec<Game>, LibraryError> {
        // let do_hard_refresh = hard_fresh.unwrap_or(false);
        if
        /* !do_hard_refresh &&*/
        let Ok(library) = get_cached_object("library") {
            return Ok(library);
        }

        let client = DROP_CLIENT_ASYNC.clone();
        let response = generate_url(&["/api/v1/client/user/library"], &[], self.base_url)?;
        let response = client
            .get(response)
            .header("Authorization", generate_authorization_header(self.auth))
            .send()
            .await
            .map_err(|e| LibraryError::FetchError(RemoteAccessError::FetchError(e.into())))?;

        if response.status() != 200 {
            let err = response.json().await.unwrap_or(DropServerError {
                status_code: 500,
                status_message: "Invalid response from server.".to_owned(),
            });
            warn!("{err:?}");
            return Err(LibraryError::FetchError(
                RemoteAccessError::InvalidResponse(err),
            ));
        }

        let mut games: Vec<Game> = response
            .json()
            .await
            .map_err(|e| RemoteAccessError::FetchError(e.into()))?;

        let mut handle = state.lock();

        let mut db_handle = borrow_db_mut_checked();

        for game in &games {
            handle.games_mut().insert(game.id().clone(), game.clone());
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
    async fn get_collections(&self) -> Result<Vec<Collection>, LibraryError> {
        todo!()
    }

    fn install(&mut self, game_id: String) {
        todo!()
    }

    fn uninstall(&mut self, game_id: String) {
        todo!()
    }

    fn metadata(&self) -> LibraryProviderMetadata {
        todo!()
    }
}

async fn fetch_library_logic(state: &Mutex<AppState>) -> Result<Vec<Game>, RemoteAccessError> {}
async fn fetch_library_logic_offline() -> Result<Vec<Game>, RemoteAccessError> {
    todo!()
}
