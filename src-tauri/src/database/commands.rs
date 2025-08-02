use std::{
    fs::create_dir_all,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{
    database::{db::borrow_db_mut_checked, scan::scan_install_dirs}, error::download_manager_error::DownloadManagerError,
};

use super::{
    db::{borrow_db_checked, DATA_ROOT_DIR},
    debug::SystemData,
    models::data::Settings,
};

// Will, in future, return disk/remaining size
// Just returns the directories that have been set up
#[tauri::command]
pub fn fetch_download_dir_stats() -> Vec<PathBuf> {
    let lock = borrow_db_checked();
    lock.applications.install_dirs.clone()
}

#[tauri::command]
pub fn delete_download_dir(index: usize) {
    let mut lock = borrow_db_mut_checked();
    lock.applications.install_dirs.remove(index);
}

#[tauri::command]
pub fn add_download_dir(new_dir: PathBuf) -> Result<(), DownloadManagerError<()>> {
    // Check the new directory is all good
    let new_dir_path = Path::new(&new_dir);
    if new_dir_path.exists() {
        let dir_contents = new_dir_path.read_dir()?;
        if dir_contents.count() != 0 {
            return Err(Error::new(
                ErrorKind::DirectoryNotEmpty,
                "Selected directory cannot contain any existing files",
            )
            .into());
        }
    } else {
        create_dir_all(new_dir_path)?;
    }

    // Add it to the dictionary
    let mut lock = borrow_db_mut_checked();
    if lock.applications.install_dirs.contains(&new_dir) {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            "Selected directory already exists in database",
        )
        .into());
    }
    lock.applications.install_dirs.push(new_dir);
    drop(lock);

    scan_install_dirs();

    Ok(())
}

#[tauri::command]
pub fn update_settings(new_settings: Value) {
    let mut db_lock = borrow_db_mut_checked();
    let mut current_settings = serde_json::to_value(db_lock.settings.clone()).unwrap();
    for (key, value) in new_settings.as_object().unwrap() {
        current_settings[key] = value.clone();
    }
    let new_settings: Settings = serde_json::from_value(current_settings).unwrap();
    db_lock.settings = new_settings;
}
#[tauri::command]
pub fn fetch_settings() -> Settings {
    borrow_db_checked().settings.clone()
}
#[tauri::command]
pub fn fetch_system_data() -> SystemData {
    let db_handle = borrow_db_checked();
    SystemData::new(
        db_handle.auth.as_ref().unwrap().client_id.clone(),
        db_handle.base_url.clone(),
        DATA_ROOT_DIR.to_string_lossy().to_string(),
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    )
}
