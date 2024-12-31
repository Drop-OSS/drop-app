use std::sync::Arc;

use crate::download_manager::{download_thread_control_flag::DownloadThreadControl, downloadable::Downloadable, progress_object::ProgressObject};

pub struct ToolDownloadAgent {
    id: String,
    version: String,
    location: String,
    control_flag: DownloadThreadControl,
    progress: Arc<ProgressObject>,
}
impl Downloadable for ToolDownloadAgent {
    fn download(&mut self) -> Result<(), crate::download_manager::application_download_error::ApplicationDownloadError> {
        todo!()
    }

    fn progress(&self) -> Arc<ProgressObject> {
        todo!()
    }

    fn control_flag(&self) -> DownloadThreadControl {
        todo!()
    }

    fn metadata(&self) -> crate::download_manager::downloadable_metadata::DownloadableMetadata {
        todo!()
    }

    fn on_error(&self) {
        todo!()
    }

    fn on_complete(&self) {
        todo!()
    }
}