use std::{mem::ManuallyDrop, sync::LazyLock};

use log::error;

use crate::db::{DBRead, DBWrite, DatabaseImpls, DatabaseInterface};

pub mod db;
pub mod debug;
pub mod models;
pub mod process;
pub mod runtime_models;
pub mod drop_data;

pub static DB: LazyLock<DatabaseInterface> = LazyLock::new(DatabaseInterface::set_up_database);

pub fn borrow_db_checked<'a>() -> DBRead<'a> {
    match DB.borrow_data() {
        Ok(data) => DBRead(data),
        Err(e) => {
            error!("database borrow failed with error {e}");
            panic!("database borrow failed with error {e}");
        }
    }
}

pub fn borrow_db_mut_checked<'a>() -> DBWrite<'a> {
    match DB.borrow_data_mut() {
        Ok(data) => DBWrite(ManuallyDrop::new(data)),
        Err(e) => {
            error!("database borrow mut failed with error {e}");
            panic!("database borrow mut failed with error {e}");
        }
    }
}
