use std::{path::PathBuf, sync::LazyLock};

#[cfg(not(debug_assertions))]
pub const DATA_ROOT_PREFIX: &str = "drop";
#[cfg(debug_assertions)]
pub const DATA_ROOT_PREFIX: &str = "drop-debug";

pub const DROP_DATA_PATH: &str = ".dropdata";

pub const RETRY_COUNT: usize = 3;

pub const TARGET_BUCKET_SIZE: usize = 63 * 1000 * 1000;
pub const MAX_FILES_PER_BUCKET: usize = (1024 / 4) - 1;

pub const UMU_BASE_LAUNCHER_EXECUTABLE: &str = "umu-run";
pub const UMU_INSTALL_DIRS: [&str; 4] = ["/app/share", "/use/local/share", "/usr/share", "/opt"];
