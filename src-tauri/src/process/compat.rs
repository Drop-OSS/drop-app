use std::{
    fs::create_dir_all,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::db::DATA_ROOT_DIR;

pub struct CompatibilityManager {
    compat_tools_path: PathBuf,
    prefixes_path: PathBuf,
    created_paths: AtomicBool,
}

/*
This gets built into both the Windows & Linux client, but
we only need it in the Linux client. Therefore, it should
do nothing but take a little bit of memory if we're on
Windows.
*/
impl CompatibilityManager {
    pub fn new() -> Self {
        let root_dir_lock = DATA_ROOT_DIR.lock().unwrap();
        let compat_tools_path = root_dir_lock.join("compatibility_tools");
        let prefixes_path = root_dir_lock.join("prefixes");
        drop(root_dir_lock);

        Self {
            compat_tools_path,
            prefixes_path,
            created_paths: AtomicBool::new(false),
        }
    }

    fn ensure_paths_exist(&self) -> Result<(), String> {
        if self.created_paths.fetch_and(true, Ordering::Relaxed) {
            return Ok(());
        }
        if !self.compat_tools_path.exists() {
            create_dir_all(self.compat_tools_path.clone()).map_err(|e| e.to_string())?;
        }
        if !self.prefixes_path.exists() {
            create_dir_all(self.prefixes_path.clone()).map_err(|e| e.to_string())?;
        }
        self.created_paths.store(true, Ordering::Relaxed);

        Ok(())
    }

    
}
