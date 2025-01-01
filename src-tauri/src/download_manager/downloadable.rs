use std::sync::Arc;

use tauri::AppHandle;

use super::{
    application_download_error::ApplicationDownloadError, download_thread_control_flag::DownloadThreadControl, downloadable_metadata::DownloadableMetadata, progress_object::ProgressObject
};

pub trait Downloadable: Send + Sync {
    fn download(&self) -> Result<bool, ApplicationDownloadError>;
    fn progress(&self) -> Arc<ProgressObject>;
    fn control_flag(&self) -> DownloadThreadControl;
    fn metadata(&self) -> Arc<DownloadableMetadata>;
    fn on_initialised(&self, app_handle: &AppHandle);
    fn on_error(&self, app_handle: &AppHandle);
    fn on_complete(&self, app_handle: &AppHandle);
    fn on_incomplete(&self, app_handle: &AppHandle);
}
