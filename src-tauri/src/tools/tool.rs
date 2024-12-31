use std::{path::PathBuf, sync::Arc};

use rustix::path::Arg;

use crate::download_manager::{download_thread_control_flag::DownloadThreadControl, downloadable::Downloadable, progress_object::ProgressObject};

pub struct ToolDownloader {
    id: String,
    version: String,
    location: Option<PathBuf>,
    progress: Arc<ProgressObject>,
    control_flag: DownloadThreadControl
}
impl Downloadable for ToolDownloader {
    fn get_progress_object(&self) -> std::sync::Arc<crate::download_manager::progress_object::ProgressObject> {
        todo!()
    }

    fn version(&self) -> String {
        self.version.clone()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn download(&mut self) -> Result<(), crate::download_manager::application_download_error::ApplicationDownloadError> {
        todo!()
    }

    fn progress(&self) -> Arc<ProgressObject> {
        self.progress.clone()
    }

    fn control_flag(&self) -> DownloadThreadControl {
        self.control_flag.clone()
    }

    fn install_dir(&self) -> String {
        self.location.clone().unwrap().to_string_lossy().to_string()
    }
}