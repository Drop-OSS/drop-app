#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]

use std::sync::{LazyLock, nonpoison::Mutex};

use client::app_state::AppState;
use database::{borrow_db_checked, models::{Game, LibraryProviderMetadata, ProviderType}};
use futures::{StreamExt, future::join_all};
use itertools::Itertools;

use crate::{drop::drop::DropLibraryProvider, error::LibraryError, provider::LibraryProvider};

pub mod drop;
pub mod error;
pub mod provider;

pub static LIBRARY: LazyLock<Library> = LazyLock::new(Library::init);

pub struct Library {
    providers: Vec<Box<dyn LibraryProvider>>,
}

impl Library {
    pub fn init() -> Self {
        let metadata = borrow_db_checked();
        let library = &metadata.library;
        let providers = library.providers().iter().map(|provider| {
            Library::construct(provider)
        }).collect();
        Self {
            providers
        }
    }
    fn construct(provider: &LibraryProviderMetadata) -> Box<dyn LibraryProvider> {
        todo!()
    }
    pub async fn get_library(
        &self,
        state: &tauri::State<'_, Mutex<AppState>>,
    ) -> (Vec<Game>, Vec<LibraryError>) {
        let res = join_all(
            self.providers
                .iter()
                .map(|provider| provider.get_library(state)),
        )
        .await
        .into_iter()
        .fold(
            (Vec::new(), Vec::new()),
            |(mut acc_ok, mut acc_err), res| {
                match res {
                    Ok(games) => acc_ok.extend(games),
                    Err(e) => acc_err.push(e),
                };
                (acc_ok, acc_err)
            },
        );
        res
    }
    pub fn add(&mut self, provider: LibraryProviderMetadata) {
        let new_provider = Box::new(match provider.provider() {
            ProviderType::Drop(_) => DropLibraryProvider::new(provider),
        });
        self.providers.push(new_provider);
    }
    pub fn remove(&mut self, id: usize) {
        self.providers.retain(|v| v.metadata().id() != id);
    }
}
