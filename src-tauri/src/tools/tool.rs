use std::path::PathBuf;

use super::external_component::ExternalComponent;

pub struct Tool {
    name: String,
    version: String,
    location: Option<PathBuf>,
}
impl ExternalComponent for Tool {
    fn download(&mut self) {
        todo!()
    }

    fn version(&self) -> &String {
        &self.version
    }

    fn is_installed(&self) -> bool {
        self.location.is_some()
    }

    fn location(&self) -> &Option<PathBuf> {
        &self.location
    }
}