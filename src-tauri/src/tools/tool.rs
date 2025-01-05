use std::sync::Arc;

use tauri::AppHandle;

use crate::download_manager::{
    application_download_error::ApplicationDownloadError,
    download_thread_control_flag::DownloadThreadControl, downloadable::Downloadable,
    downloadable_metadata::DownloadableMetadata, progress_object::ProgressObject,
};

#[allow(unused)]
pub struct ToolDownloadAgent {
    id: String,
    version: String,
    location: String,
    control_flag: DownloadThreadControl,
    progress: Arc<ProgressObject>,
}
#[allow(unused)]
impl Downloadable for ToolDownloadAgent {
    fn download(&self, app_handle: &AppHandle) -> Result<bool, ApplicationDownloadError> {
        todo!()
    }

    fn progress(&self) -> Arc<ProgressObject> {
        todo!()
    }

    fn control_flag(&self) -> DownloadThreadControl {
        todo!()
    }

    fn status(&self) -> crate::download_manager::download_manager::DownloadStatus {
        todo!()
    }

    fn metadata(&self) -> DownloadableMetadata {
        todo!()
    }

    fn on_initialised(&self, app_handle: &tauri::AppHandle) {
        todo!()
    }

    fn on_error(
        &self,
        app_handle: &tauri::AppHandle,
        error: crate::download_manager::application_download_error::ApplicationDownloadError,
    ) {
        todo!()
    }

    fn on_complete(&self, app_handle: &tauri::AppHandle) {
        todo!()
    }

    fn on_incomplete(&self, app_handle: &tauri::AppHandle) {
        todo!()
    }

    fn on_cancelled(&self, app_handle: &tauri::AppHandle) {
        todo!()
    }
}
