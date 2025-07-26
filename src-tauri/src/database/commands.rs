use std::{
    fs::create_dir_all,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{
    database::db::borrow_db_mut_checked, error::download_manager_error::DownloadManagerError,
};

use super::{
    db::{DATA_ROOT_DIR, borrow_db_checked},
    debug::SystemData,
    models::data::Settings,
};

// Will, in future, return disk/remaining size
// Just returns the directories that have been set up
#[tauri::command]
pub async fn fetch_download_dir_stats() -> Vec<PathBuf> {
    let lock = borrow_db_checked().await;
    lock.applications.install_dirs.clone()
}

#[tauri::command]
pub async fn delete_download_dir(index: usize) {
    let mut lock = borrow_db_mut_checked().await;
    lock.applications.install_dirs.remove(index);
}

#[tauri::command]
pub async fn add_download_dir(new_dir: PathBuf) -> Result<(), DownloadManagerError<()>> {
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
    let mut lock = borrow_db_mut_checked().await;
    if lock.applications.install_dirs.contains(&new_dir) {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            "Selected directory already exists in database",
        )
        .into());
    }
    lock.applications.install_dirs.push(new_dir);
    drop(lock);

    Ok(())
}

#[tauri::command]
pub async fn update_settings(new_settings: Value) {
    let mut db_lock = borrow_db_mut_checked().await;
    let mut current_settings = serde_json::to_value(db_lock.settings.clone()).unwrap();
    for (key, value) in new_settings.as_object().unwrap() {
        current_settings[key] = value.clone();
    }
    let new_settings: Settings = serde_json::from_value(current_settings).unwrap();
    db_lock.settings = new_settings;
}
#[tauri::command]
pub async fn fetch_settings() -> Settings {
    borrow_db_checked().await.settings.clone()
}
#[tauri::command]
pub async fn fetch_system_data() -> SystemData {
    let db_handle = borrow_db_checked().await;
    SystemData::new(
        db_handle.auth.as_ref().unwrap().client_id.clone(),
        db_handle.base_url.clone(),
        DATA_ROOT_DIR.to_string_lossy().to_string(),
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    )
}
