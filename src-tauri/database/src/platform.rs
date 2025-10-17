use serde::{Deserialize, Serialize};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Platform {
    Windows,
    Linux,
    macOS,
}

impl Platform {
    #[cfg(target_os = "windows")]
    pub const HOST: Platform = Self::Windows;
    #[cfg(target_os = "macos")]
    pub const HOST: Platform = Self::macOS;
    #[cfg(target_os = "linux")]
    pub const HOST: Platform = Self::Linux;

    pub fn is_case_sensitive(&self) -> bool {
        match self {
            Self::Windows | Self::macOS => false,
            Self::Linux => true,
        }
    }
}

impl From<&str> for Platform {
    fn from(value: &str) -> Self {
        match value.to_lowercase().trim() {
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "mac" | "macos" => Self::macOS,
            _ => unimplemented!(),
        }
    }
}

impl From<whoami::Platform> for Platform {
    fn from(value: whoami::Platform) -> Self {
        match value {
            whoami::Platform::Windows => Platform::Windows,
            whoami::Platform::Linux => Platform::Linux,
            whoami::Platform::MacOS => Platform::macOS,
            platform => unimplemented!("Playform {} is not supported", platform),
        }
    }
}
