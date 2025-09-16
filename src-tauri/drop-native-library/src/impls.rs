use drop_library::{
    errors::DropLibraryError, game::{LibraryGame, LibraryGamePreview}, libraries::{DropLibraryProvider, LibraryFetchConfig, LibraryProviderIdentifier}
};
use drop_remote::{fetch_object::fetch_object, DropRemoteContext};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
pub struct DropNativeLibraryProvider {
    identifier: LibraryProviderIdentifier,
    context: Option<DropRemoteContext>,
}

impl DropNativeLibraryProvider {
    pub fn configure(&mut self, base_url: Url) {
        self.context = Some(DropRemoteContext::new(base_url));
    }
}

impl DropLibraryProvider for DropNativeLibraryProvider {
    fn build(identifier: LibraryProviderIdentifier) -> Self {
        Self {
            identifier,
            context: None,
        }
    }

    fn id(&self) -> &LibraryProviderIdentifier {
        &self.identifier
    }
    
    async fn load_object(&self, request: tauri::http::Request<Vec<u8>>, responder: tauri::UriSchemeResponder) -> Result<(), DropLibraryError> {
        let context = self.context.as_ref().ok_or(DropLibraryError::Unconfigured)?;
        fetch_object(context, request, responder).await;
        Ok(())
    }
    
    async fn fetch_library(
        &self,
        config: &LibraryFetchConfig
    ) -> Result<Vec<LibraryGamePreview>, DropLibraryError> {
        todo!()
    }
    
    async fn fetch_game(&self, config: &LibraryFetchConfig) -> Result<LibraryGame, DropLibraryError> {
        todo!()
    }

    
}
