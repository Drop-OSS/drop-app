use std::path::PathBuf;

pub trait ExternalComponent {
    fn download(&mut self);
    fn version(&self) -> &String;
    fn is_installed(&self) -> bool;
    fn location(&self) -> &Option<PathBuf>;
}