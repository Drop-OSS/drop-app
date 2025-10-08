#![feature(nonpoison_rwlock)]

pub mod db;
pub mod debug;
pub mod models;
pub mod platform;
pub mod interface;

pub use models::data::{
    ApplicationTransientStatus,
    Database,
    DatabaseApplications,
    DatabaseAuth,
    DownloadType,
    DownloadableMetadata,
    GameDownloadStatus,
    GameVersion,
    Settings
};
pub use db::DB;
pub use interface::{borrow_db_checked, borrow_db_mut_checked};
