use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

pub struct ProcessManager {
    current_platform: Platform,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            current_platform: if cfg!(windows) {
                Platform::Windows
            } else {
                Platform::Linux
            },
        }
    }

    pub fn valid_platform(&self, platform: &Platform) -> Result<bool, String> {
        let current = &self.current_platform;
        let valid_platforms = PROCESS_COMPATABILITY_MATRIX
            .get(current)
            .ok_or("Incomplete platform compatability matrix.")?;

        Ok(valid_platforms.contains(platform))
    }
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub enum Platform {
    Windows,
    Linux,
}

pub type ProcessCompatabilityMatrix = HashMap<Platform, Vec<Platform>>;
pub static PROCESS_COMPATABILITY_MATRIX: LazyLock<ProcessCompatabilityMatrix> =
    LazyLock::new(|| {
        let mut matrix: ProcessCompatabilityMatrix = HashMap::new();

        matrix.insert(Platform::Windows, vec![Platform::Windows]);
        matrix.insert(Platform::Linux, vec![Platform::Linux]); // TODO: add Proton support

        return matrix;
    });
