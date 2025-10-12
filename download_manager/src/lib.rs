#![feature(duration_millis_float)]
#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]

use std::sync::{nonpoison::Mutex, LazyLock};

use crate::{download_manager_builder::DownloadManagerBuilder, download_manager_frontend::DownloadManager};

pub mod download_manager_builder;
pub mod download_manager_frontend;
pub mod downloadable;
pub mod util;
pub mod error;
pub mod frontend_updates;

pub static DOWNLOAD_MANAGER: LazyLock<Mutex<DownloadManager>> = LazyLock::new(|| todo!());