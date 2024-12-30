use std::sync::Arc;

use crate::downloads::download_agent::GameDownloadError;

use super::{
    download_thread_control_flag::DownloadThreadControl,
    progress_object::ProgressObject,
};

pub trait Downloadable: Sync {
    fn get_progress_object(&self) -> Arc<ProgressObject>;
    fn version(&self) -> String;
    fn id(&self) -> String;
    fn download(&mut self) -> Result<(), GameDownloadError>;
    fn progress(&self) -> Arc<ProgressObject>;
    fn control_flag(&self) -> DownloadThreadControl;
    fn install_dir(&self) -> String;
}
