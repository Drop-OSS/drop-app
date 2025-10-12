use std::sync::Arc;

use database::DownloadableMetadata;
use tauri::AppHandle;

use crate::error::ApplicationDownloadError;

use super::{
    download_manager_frontend::DownloadStatus,
    util::{download_thread_control_flag::DownloadThreadControl, progress_object::ProgressObject},
};

/**
 * Downloadables are responsible for managing their specific object's download state
 * e.g, the GameDownloadAgent is responsible for pushing game updates
 *
 * But the download manager manages the queue state
 */
pub trait Downloadable: Send + Sync {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError>;
    fn validate(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError>;

    fn progress(&self) -> Arc<ProgressObject>;
    fn control_flag(&self) -> DownloadThreadControl;
    fn status(&self) -> DownloadStatus;
    fn metadata(&self) -> DownloadableMetadata;
    fn on_queued(&self, app_handle: &AppHandle);
    fn on_error(&self, app_handle: &AppHandle, error: &ApplicationDownloadError);
    fn on_complete(&self, app_handle: &AppHandle);
    fn on_cancelled(&self, app_handle: &AppHandle);
}
