use std::sync::LazyLock;

use crate::process::process_manager::Platform;

pub const ALL: &[&str] = &[
    ROOT,
    GAME,
    BASE,
    HOME,
    STORE_USER_ID,
    OS_USER_NAME,
    WIN_APP_DATA,
    WIN_LOCAL_APP_DATA,
    WIN_DOCUMENTS,
    WIN_PUBLIC,
    WIN_PROGRAM_DATA,
    WIN_DIR,
    XDG_DATA,
    XDG_CONFIG,
];

/// These are paths where `<placeholder>/*/` is suspicious.
pub const AVOID_WILDCARDS: &[&str] = &[
    ROOT,
    HOME,
    WIN_APP_DATA,
    WIN_LOCAL_APP_DATA,
    WIN_DOCUMENTS,
    WIN_PUBLIC,
    WIN_PROGRAM_DATA,
    WIN_DIR,
    XDG_DATA,
    XDG_CONFIG,
];

pub const ROOT: &str = "<root>";
pub const GAME: &str = "<game>";
pub const BASE: &str = "<base>";
pub const HOME: &str = "<home>";
pub const STORE_USER_ID: &str = "<storeUserId>";
pub const OS_USER_NAME: &str = "<osUserName>";
pub const WIN_APP_DATA: &str = "<winAppData>";
pub const WIN_LOCAL_APP_DATA: &str = "<winLocalAppData>";
pub const WIN_LOCAL_APP_DATA_LOW: &str = "<winLocalAppDataLow>";
pub const WIN_DOCUMENTS: &str = "<winDocuments>";
pub const WIN_PUBLIC: &str = "<winPublic>";
pub const WIN_PROGRAM_DATA: &str = "<winProgramData>";
pub const WIN_DIR: &str = "<winDir>";
pub const XDG_DATA: &str = "<xdgData>";
pub const XDG_CONFIG: &str = "<xdgConfig>";
pub const SKIP: &str = "<skip>";

pub static OS_USERNAME: LazyLock<String> = LazyLock::new(|| whoami::username());

