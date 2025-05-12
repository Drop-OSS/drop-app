use std::{fs::File, io::Write, path::{Component, PathBuf}};

use super::{backup_manager::BackupHandler, conditions::Condition, metadata::GameFile, placeholder::*};
use log::warn;
use rustix::path::Arg;
use tempfile::tempfile;

use crate::{
    database::db::GameVersion, error::backup_error::BackupError, process::process_manager::Platform,
};

use super::{backup_manager::BackupManager, metadata::CloudSaveMetadata, normalise::normalize};

pub fn resolve(meta: &mut CloudSaveMetadata) -> File {
    let f = File::create_new("save").unwrap();
    let compressor = zstd::Encoder::new(f, 22).unwrap();
    let mut tarball = tar::Builder::new(compressor);
    let manager = BackupManager::new();
    for file in meta.files.iter_mut() {
        let id = uuid::Uuid::new_v4().to_string();
        let os = match file
            .conditions
            .iter()
            .find_map(|p| match p {
                super::conditions::Condition::Os(os) => Some(os),
                _ => None,
            })
            .cloned()
        {
            Some(os) => os,
            None => {
                warn!(
                    "File {:?} could not be backed up because it did not provide an OS",
                    &file
                );
                continue;
            }
        };
        let handler = match manager.sources.get(&(manager.current_platform, os)) {
            Some(h) => *h,
            None => continue,
        };
        let t_path = PathBuf::from(normalize(&file.path, os));
        println!("{:?}", &t_path);
        let path = parse_path(t_path, handler, &meta.game_version).unwrap();
        let f = std::fs::metadata(&path).unwrap(); // TODO: Fix unwrap here
        if f.is_dir() {
            tarball.append_dir(&id, path).unwrap();
        } else if f.is_file() {
            tarball
                .append_file(&id, &mut File::open(path).unwrap())
                .unwrap();
        }
        file.id = Some(id);
    }
    let binding = serde_json::to_string(meta).unwrap();
    println!("Binding: {}", &binding);
    let serialized = binding.as_bytes();
    let mut file = tempfile().unwrap();
    file.write(serialized).unwrap();
    tarball.append_file("metadata", &mut file).unwrap();
    tarball.into_inner().unwrap().finish().unwrap()
}

pub fn parse_path(
    path: PathBuf,
    backup_handler: &dyn BackupHandler,
    game: &GameVersion,
) -> Result<PathBuf, BackupError> {
    println!("Parsing: {:?}", &path);
    let mut s = PathBuf::new();
    for component in path.components() {
        match component.as_str().unwrap() {
            ROOT => { s.push(backup_handler.root_translate(&path, game)?)},
            GAME => { s.push(backup_handler.game_translate(&path, game)?)},
            BASE => { s.push(backup_handler.base_translate(&path, game)?)},
            HOME => { s.push(backup_handler.home_translate(&path, game)?)},
            STORE_USER_ID => { s.push(backup_handler.store_user_id_translate(&path, game)?)},
            OS_USER_NAME => { s.push(backup_handler.os_user_name_translate(&path, game)?)},
            WIN_APP_DATA => { s.push(backup_handler.win_app_data_translate(&path, game)?)},
            WIN_LOCAL_APP_DATA => { s.push(backup_handler.win_local_app_data_translate(&path, game)?)},
            WIN_LOCAL_APP_DATA_LOW => { s.push(backup_handler.win_local_app_data_low_translate(&path, game)?)},
            WIN_DOCUMENTS => { s.push(backup_handler.win_documents_translate(&path, game)?)},
            WIN_PUBLIC => { s.push(backup_handler.win_public_translate(&path, game)?)},
            WIN_PROGRAM_DATA => { s.push(backup_handler.win_program_data_translate(&path, game)?)},
            WIN_DIR => { s.push(backup_handler.win_dir_translate(&path, game)?)},
            XDG_DATA => { s.push(backup_handler.xdg_data_translate(&path, game)?)},
            XDG_CONFIG => { s.push(backup_handler.xdg_config_translate(&path, game)?)},
            SKIP => {  },
            _ => s.push(PathBuf::from(component.as_os_str()))
        }
    }

    println!("Final line: {:?}", &s);
    Ok(s)
}

pub fn test() {
    let mut meta = CloudSaveMetadata {
        files: vec![GameFile {
            path: String::from("<home>/favicon.png"),
            id: None,
            data_type: super::metadata::DataType::File,
            tags: Vec::new(),
            conditions: vec![Condition::Os(Platform::Linux)],
        }],
        game_version: GameVersion {
            game_id: String::new(),
            version_name: String::new(),
            platform: Platform::Linux,
            launch_command: String::new(),
            launch_args: Vec::new(),
            launch_command_template: String::new(),
            setup_command: String::new(),
            setup_args: Vec::new(),
            setup_command_template: String::new(),
            only_setup: true,
            version_index: 0,
            delta: false,
            umu_id_override: None,
        },
        save_id: String::from("aaaaaaa"),
    };
    let file = resolve(&mut meta);
}
