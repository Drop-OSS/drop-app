use std::sync::Arc;

use tauri::AppHandle;

use crate::error::application_download_error::ApplicationDownloadError;

use super::{
    download_manager::DownloadStatus, download_thread_control_flag::DownloadThreadControl,
    downloadable_metadata::DownloadableMetadata, progress_object::ProgressObject,
};

pub trait Downloadable: Send + Sync {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError>;
    fn progress(&self) -> Arc<ProgressObject>;
    fn control_flag(&self) -> DownloadThreadControl;
    fn status(&self) -> DownloadStatus;
    fn metadata(&self) -> DownloadableMetadata;
    fn on_initialised(&self, app_handle: &AppHandle);
    fn on_error(&self, app_handle: &AppHandle, error: &ApplicationDownloadError);
    fn on_complete(&self, app_handle: &AppHandle);
    fn on_incomplete(&self, app_handle: &AppHandle);
    fn on_cancelled(&self, app_handle: &AppHandle);
}
