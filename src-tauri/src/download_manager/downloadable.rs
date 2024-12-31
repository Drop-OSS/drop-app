use std::sync::Arc;

use super::{
    application_download_error::ApplicationDownloadError, download_thread_control_flag::DownloadThreadControl, progress_object::ProgressObject
};

pub trait Downloadable: Send + Sync {
    fn version(&self) -> String;
    fn id(&self) -> String;
    fn download(&mut self) -> Result<(), ApplicationDownloadError>;
    fn progress(&self) -> Arc<ProgressObject>;
    fn control_flag(&self) -> DownloadThreadControl;
    fn install_dir(&self) -> String;
}
