use std::sync::nonpoison::Mutex;

use async_trait::async_trait;
use client::app_state::AppState;
use database::models::{Collection, Game, LibraryProviderMetadata};

use crate::error::LibraryError;

#[async_trait]
pub trait LibraryProvider: Sync + Send {
    async fn get_library(&self, state: &tauri::State<'_, Mutex<AppState>>) -> Result<Vec<Game>, LibraryError>;
    async fn get_collections(&self) -> Result<Vec<Collection>, LibraryError>;
    fn install(&mut self, game_id: String);
    fn uninstall(&mut self, game_id: String);
    fn metadata(&self) -> LibraryProviderMetadata;
}