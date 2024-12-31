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
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn progress(&self) -> Arc<ProgressObject> {
        self.progress.clone()
    }
    
    fn control_flag(&self) -> DownloadThreadControl {
        self.control_flag.clone()
    }
    
    fn install_dir(&self) -> String {
        self.location.clone()
    }
    
    fn version(&self) -> String {
        self.version.clone()
    }
    
    fn download(&mut self) -> Result<(), crate::download_manager::application_download_error::ApplicationDownloadError> {
        todo!()
    }
}