use std::{collections::HashMap, env, path::Path, str::FromStr as _, sync::{Arc, Mutex}};

use drop_database::{borrow_db_checked, borrow_db_mut_checked, db::{DatabaseImpls as _, DATA_ROOT_DIR}, models::data::GameDownloadStatus, DB};
use drop_downloads::download_manager_builder::DownloadManagerBuilder;
use drop_process::process_manager::ProcessManager;
use log::{debug, info, warn, LevelFilter};
use log4rs::{append::{console::ConsoleAppender, file::FileAppender}, config::{Appender, Root}, encode::pattern::PatternEncoder, Config};
use tauri::AppHandle;

use crate::{auth, client::autostart::sync_autostart_on_startup, database::scan::scan_install_dirs, AppState, AppStatus};

pub async fn setup(handle: AppHandle) -> AppState {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d} | {l} | {f}:{L} - {m}{n}",
        )))
        .append(false)
        .build(DATA_ROOT_DIR.join("./drop.log"))
        .unwrap();

    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d} | {l} | {f}:{L} - {m}{n}",
        )))
        .build();

    let log_level = env::var("RUST_LOG").unwrap_or(String::from("Info"));

    let config = Config::builder()
        .appenders(vec![
            Appender::builder().build("logfile", Box::new(logfile)),
            Appender::builder().build("console", Box::new(console)),
        ])
        .build(
            Root::builder()
                .appenders(vec!["logfile", "console"])
                .build(LevelFilter::from_str(&log_level).expect("Invalid log level")),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    let download_manager = Arc::new(DownloadManagerBuilder::build(handle.clone()));
    let process_manager = Box::leak(Box::new(Mutex::new(ProcessManager::new(handle.clone()))));

    debug!("checking if database is set up");
    let is_set_up = DB.database_is_set_up();

    scan_install_dirs();

    if !is_set_up {
        return AppState {
            status: AppStatus::NotConfigured,
            user: None,
            download_manager,
            process_manager,
        };
    }

    debug!("database is set up");

    // TODO: Account for possible failure
    let (app_status, user) = auth::setup().await;

    let db_handle = borrow_db_checked();
    let mut missing_games = Vec::new();
    let statuses = db_handle.applications.game_statuses.clone();
    drop(db_handle);

    for (game_id, status) in statuses {
        match status {
            GameDownloadStatus::Remote {} => {}
            GameDownloadStatus::PartiallyInstalled { .. } => {}
            GameDownloadStatus::SetupRequired {
                version_name: _,
                install_dir,
            } => {
                let install_dir_path = Path::new(&install_dir);
                if !install_dir_path.exists() {
                    missing_games.push(game_id);
                }
            }
            GameDownloadStatus::Installed {
                version_name: _,
                install_dir,
            } => {
                let install_dir_path = Path::new(&install_dir);
                if !install_dir_path.exists() {
                    missing_games.push(game_id);
                }
            }
        }
    }

    info!("detected games missing: {missing_games:?}");

    let mut db_handle = borrow_db_mut_checked();
    for game_id in missing_games {
        db_handle
            .applications
            .game_statuses
            .entry(game_id)
            .and_modify(|v| *v = GameDownloadStatus::Remote {});
    }

    drop(db_handle);

    debug!("finished setup!");

    // Sync autostart state
    if let Err(e) = sync_autostart_on_startup(&handle) {
        warn!("failed to sync autostart state: {e}");
    }

    AppState {
        status: app_status,
        user,
        download_manager,
        process_manager,
    }
}
