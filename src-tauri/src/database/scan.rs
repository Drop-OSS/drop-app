use std::fs;

use log::warn;

use crate::{
    database::{
        db::{borrow_db_checked, borrow_db_mut_checked},
        models::data::{
            v1::{DownloadType, DownloadableMetadata},
            v2::GameDownloadStatus,
        },
    },
    games::{
        downloads::drop_data::{v1::DropData, DROP_DATA_PATH},
        library::{set_partially_installed, set_partially_installed_db},
    },
};

pub fn scan_install_dirs() {
    let mut db_lock = borrow_db_mut_checked();
    for install_dir in db_lock.applications.install_dirs.clone() {
        let Ok(files) = fs::read_dir(install_dir) else {
            continue;
        };
        for game in files.into_iter().filter(|e| e.is_ok()).map(|e| e.unwrap()) {
            let drop_data_file = game.path().join(DROP_DATA_PATH);
            if !drop_data_file.exists() {
                continue;
            }
            let game_id = game.file_name().into_string().unwrap();
            let Ok(drop_data) = DropData::read(&game.path()) else {
                warn!(
                    ".dropdata exists for {}, but couldn't read it. is it corrupted?",
                    game.file_name().into_string().unwrap()
                );
                continue;
            };
            if db_lock.applications.game_statuses.contains_key(&game_id) {
                continue;
            }

            let metadata = DownloadableMetadata::new(
                drop_data.game_id,
                Some(drop_data.game_version),
                DownloadType::Game,
            );
            set_partially_installed_db(
                &mut db_lock,
                &metadata,
                drop_data.base_path.to_str().unwrap().to_string(),
                None,
            );
        }
    }
}
