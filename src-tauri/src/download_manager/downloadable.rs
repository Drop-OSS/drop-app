use std::sync::Arc;

use tauri::AppHandle;

use crate::{
    database::models::data::DownloadableMetadata,
    error::application_download_error::ApplicationDownloadError,
};

use super::{
    download_manager_frontend::DownloadStatus,
    util::{download_thread_control_flag::DownloadThreadControl, progress_object::ProgressObject},
};

#[async_trait::async_trait]
pub trait Downloadable: Send + Sync {
    async fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError>;
    async fn progress(&self) -> Arc<ProgressObject>;
    async fn control_flag(&self) -> DownloadThreadControl;
    async fn validate(&self) -> Result<bool, ApplicationDownloadError>;
    async fn status(&self) -> DownloadStatus;
    fn metadata(&self) -> DownloadableMetadata;
    async fn on_initialised(&self, app_handle: &AppHandle);
    async fn on_error(&self, app_handle: &AppHandle, error: &ApplicationDownloadError);
    async fn on_complete(&self, app_handle: &AppHandle);
    async fn on_incomplete(&self, app_handle: &AppHandle);
    async fn on_cancelled(&self, app_handle: &AppHandle);
}
