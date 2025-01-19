use std::{
    fs::create_dir_all,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{database::settings::Settings, download_manager::{download_manager::DownloadManagerSignal, internal_error::InternalError}, DB};

use super::{db::DATA_ROOT_DIR, debug::SystemData};

// Will, in future, return disk/remaining size
// Just returns the directories that have been set up
#[tauri::command]
pub fn fetch_download_dir_stats() -> Vec<PathBuf> {
    let lock = DB.borrow_data().unwrap();
    lock.applications.install_dirs.clone()
}

#[tauri::command]
pub fn delete_download_dir(index: usize) {
    let mut lock = DB.borrow_data_mut().unwrap();
    lock.applications.install_dirs.remove(index);
    drop(lock);
    DB.save().unwrap();
}

#[tauri::command]
pub fn add_download_dir(new_dir: PathBuf) -> Result<(), InternalError<()>> {
    // Check the new directory is all good
    let new_dir_path = Path::new(&new_dir);
    if new_dir_path.exists() {
        let dir_contents = new_dir_path.read_dir()?;
        if dir_contents.count() != 0 {
            return Err(Error::new(
                ErrorKind::DirectoryNotEmpty,
                "Selected directory cannot contain any existing files",
            ).into());
        }
    } else {
        create_dir_all(new_dir_path)?;
    }

    // Add it to the dictionary
    let mut lock = DB.borrow_data_mut().unwrap();
    if lock.applications.install_dirs.contains(&new_dir) {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            "Selected directory already exists in database",
        ).into());
    }
    lock.applications.install_dirs.push(new_dir);
    drop(lock);
    DB.save().unwrap();

    Ok(())
}

#[tauri::command]
pub fn update_settings(new_settings: Value) {
    let mut db_lock = DB.borrow_data_mut().unwrap();
    let mut current_settings = serde_json::to_value(db_lock.settings.clone()).unwrap();
    for (key, value) in new_settings.as_object().unwrap() {
        current_settings[key] = value.clone();
    }
    let new_settings: Settings = serde_json::from_value(current_settings).unwrap();
    db_lock.settings = new_settings;
    println!("New Settings: {:?}", db_lock.settings);
}

#[tauri::command]
pub fn fetch_system_data() -> SystemData {
    let db_handle = DB.borrow_data().unwrap();
    SystemData::new(
        db_handle.auth.as_ref().unwrap().client_id.clone(),
        db_handle.base_url.clone(),
        DATA_ROOT_DIR.lock().unwrap().to_string_lossy().to_string(),
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    )
}
