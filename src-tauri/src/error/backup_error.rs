use std::fmt::Display;

use serde_with::SerializeDisplay;

#[derive(Debug, SerializeDisplay, Clone, Copy)]
pub enum BackupError {
    InvalidSystem,
    NotFound,
    ParseError,
}

impl Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BackupError::InvalidSystem => "Attempted to generate path for invalid system",
            BackupError::NotFound => "Could not generate or find path",
            BackupError::ParseError => "Failed to parse path",
        };
        write!(f, "{}", s)
    }
}
