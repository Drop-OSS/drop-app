#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]

use std::sync::{LazyLock, nonpoison::Mutex};

use crate::process_manager::ProcessManager;

pub static PROCESS_MANAGER: LazyLock<Mutex<ProcessManager>> =
    LazyLock::new(|| Mutex::new(ProcessManager::new()));
pub mod error;
pub mod format;
pub mod process_handlers;
pub mod process_manager;
