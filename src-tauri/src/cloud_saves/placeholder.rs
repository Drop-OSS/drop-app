use std::sync::LazyLock;

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

pub const ROOT: &str = "<root>"; // a directory where games are installed (configured in backup tool)
pub const GAME: &str = "<game>"; // an installDir (if defined) or the game's canonical name in the manifest
pub const BASE: &str = "<base>"; // shorthand for <root>/<game> (unless overridden by store-specific rules)
pub const HOME: &str = "<home>"; // current user's home directory in the OS (~)
pub const STORE_USER_ID: &str = "<storeUserId>"; // a store-specific id from the manifest, corresponding to the root's store type
pub const OS_USER_NAME: &str = "<osUserName>"; // current user's ID in the game store
pub const WIN_APP_DATA: &str = "<winAppData>"; // current user's name in the OS
pub const WIN_LOCAL_APP_DATA: &str = "<winLocalAppData>"; // %APPDATA% on Windows
pub const WIN_LOCAL_APP_DATA_LOW: &str = "<winLocalAppDataLow>"; // %LOCALAPPDATA% on Windows
pub const WIN_DOCUMENTS: &str = "<winDocuments>"; // <home>/AppData/LocalLow on Windows
pub const WIN_PUBLIC: &str = "<winPublic>"; // <home>/Documents (f.k.a. <home>/My Documents) or a localized equivalent on Windows
pub const WIN_PROGRAM_DATA: &str = "<winProgramData>"; // %PUBLIC% on Windows
pub const WIN_DIR: &str = "<winDir>"; // %PROGRAMDATA% on Windows
pub const XDG_DATA: &str = "<xdgData>"; // %WINDIR% on Windows
pub const XDG_CONFIG: &str = "<xdgConfig>"; // $XDG_DATA_HOME on Linux
pub const SKIP: &str = "<skip>"; // $XDG_CONFIG_HOME on Linux

pub static OS_USERNAME: LazyLock<String> = LazyLock::new(|| whoami::username());