use std::fs;

use database::{DownloadType, DownloadableMetadata, borrow_db_mut_checked};
use log::warn;

use crate::{
    downloads::drop_data::{DROPDATA_PATH, DropData},
    library::set_partially_installed_db,
};

pub fn scan_install_dirs() {
    let mut db_lock = borrow_db_mut_checked();
    for install_dir in db_lock.applications.install_dirs.clone() {
        let Ok(files) = fs::read_dir(install_dir) else {
            continue;
        };
        for game in files.into_iter().flatten() {
            let drop_data_file = game.path().join(DROPDATA_PATH);
            if !drop_data_file.exists() {
                continue;
            }
            let Ok(drop_data) = DropData::read(&game.path()) else {
                warn!(
                    ".dropdata exists for {}, but couldn't read it. is it corrupted?",
                    game.file_name().display()
                );
                continue;
            };
            if db_lock.applications.game_statuses.contains_key(&drop_data.game_id) {
                continue;
            }

            let metadata = DownloadableMetadata::new(
                drop_data.game_id,
                drop_data.game_version,
                drop_data.target_platform,
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
