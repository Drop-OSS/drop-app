use std::sync::LazyLock;

use regex::Regex;
use crate::process::process_manager::Platform;

use super::placeholder::*;


pub fn normalize(path: &str, os: Platform) -> String {
    let mut path = path.trim().trim_end_matches(['/', '\\']).replace('\\', "/");

    if path == "~" || path.starts_with("~/") {
        path = path.replacen('~', HOME, 1);
    }

    static CONSECUTIVE_SLASHES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/{2,}").unwrap());
    static UNNECESSARY_DOUBLE_STAR_1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^/*])\*{2,}").unwrap());
    static UNNECESSARY_DOUBLE_STAR_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*{2,}([^/*])").unwrap());
    static ENDING_WILDCARD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(/\*)+$").unwrap());
    static ENDING_DOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(/\.)$").unwrap());
    static INTERMEDIATE_DOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(/\./)").unwrap());
    static BLANK_SEGMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(/\s+/)").unwrap());
    static APP_DATA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)%appdata%").unwrap());
    static APP_DATA_ROAMING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)%userprofile%/AppData/Roaming").unwrap());
    static APP_DATA_LOCAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)%localappdata%").unwrap());
    static APP_DATA_LOCAL_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)%userprofile%/AppData/Local/").unwrap());
    static USER_PROFILE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)%userprofile%").unwrap());
    static DOCUMENTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)%userprofile%/Documents").unwrap());

    for (pattern, replacement) in [
        (&CONSECUTIVE_SLASHES, "/"),
        (&UNNECESSARY_DOUBLE_STAR_1, "${1}*"),
        (&UNNECESSARY_DOUBLE_STAR_2, "*${1}"),
        (&ENDING_WILDCARD, ""),
        (&ENDING_DOT, ""),
        (&INTERMEDIATE_DOT, "/"),
        (&BLANK_SEGMENT, "/"),
        (&APP_DATA, WIN_APP_DATA),
        (&APP_DATA_ROAMING, WIN_APP_DATA),
        (&APP_DATA_LOCAL, WIN_LOCAL_APP_DATA),
        (&APP_DATA_LOCAL_2, &format!("{}/", WIN_LOCAL_APP_DATA)),
        (&USER_PROFILE, HOME),
        (&DOCUMENTS, WIN_DOCUMENTS),
    ] {
        path = pattern.replace_all(&path, replacement).to_string();
    }

    if os == Platform::Windows {
        let documents_2: Regex = Regex::new(r"(?i)<home>/Documents").unwrap();

        #[allow(clippy::single_element_loop)]
        for (pattern, replacement) in [(&documents_2, WIN_DOCUMENTS)] {
            path = pattern.replace_all(&path, replacement).to_string();
        }
    }

    for (pattern, replacement) in [
        ("{64BitSteamID}", STORE_USER_ID),
        ("{Steam3AccountID}", STORE_USER_ID),
    ] {
        path = path.replace(pattern, replacement);
    }

    path
}

fn too_broad(path: &str) -> bool {
    println!("Path: {}", path);
    use {BASE, HOME, ROOT, STORE_USER_ID, WIN_APP_DATA, WIN_DIR, WIN_DOCUMENTS, XDG_CONFIG, XDG_DATA};

    let path_lower = path.to_lowercase();

    for item in ALL {
        if path == *item {
            return true;
        }
    }

    for item in AVOID_WILDCARDS {
        if path.starts_with(&format!("{}/*", item)) || path.starts_with(&format!("{}/{}", item, STORE_USER_ID)) {
            return true;
        }
    }

    // These paths are present whether or not the game is installed.
    // If possible, they should be narrowed down on the wiki.
    for item in [
        format!("{}/{}", BASE, STORE_USER_ID), // because `<storeUserId>` is handled as `*`
        format!("{}/Documents", HOME),
        format!("{}/Saved Games", HOME),
        format!("{}/AppData", HOME),
        format!("{}/AppData/Local", HOME),
        format!("{}/AppData/Local/Packages", HOME),
        format!("{}/AppData/LocalLow", HOME),
        format!("{}/AppData/Roaming", HOME),
        format!("{}/Documents/My Games", HOME),
        format!("{}/Library/Application Support", HOME),
        format!("{}/Library/Application Support/UserData", HOME),
        format!("{}/Library/Preferences", HOME),
        format!("{}/.renpy", HOME),
        format!("{}/.renpy/persistent", HOME),
        format!("{}/Library", HOME),
        format!("{}/Library/RenPy", HOME),
        format!("{}/Telltale Games", HOME),
        format!("{}/config", ROOT),
        format!("{}/MMFApplications", WIN_APP_DATA),
        format!("{}/RenPy", WIN_APP_DATA),
        format!("{}/RenPy/persistent", WIN_APP_DATA),
        format!("{}/win.ini", WIN_DIR),
        format!("{}/SysWOW64", WIN_DIR),
        format!("{}/My Games", WIN_DOCUMENTS),
        format!("{}/Telltale Games", WIN_DOCUMENTS),
        format!("{}/unity3d", XDG_CONFIG),
        format!("{}/unity3d", XDG_DATA),
        "C:/Program Files".to_string(),
        "C:/Program Files (x86)".to_string(),
    ] {
        let item = item.to_lowercase();
        if path_lower == item
            || path_lower.starts_with(&format!("{}/*", item))
            || path_lower.starts_with(&format!("{}/{}", item, STORE_USER_ID.to_lowercase()))
            || path_lower.starts_with(&format!("{}/savesdir", item))
        {
            return true;
        }
    }
    

    // Drive letters:
    let drives: Regex = Regex::new(r"^[a-zA-Z]:$").unwrap();
    if drives.is_match(path) {
        return true;
    }

    // Colon not for a drive letter
    if path.get(2..).is_some_and(|path| path.contains(':')) {
        return true;
    }

    // Root:
    if path == "/" {
        return true;
    }

    // Relative path wildcard:
    if path.starts_with('*') {
        return true;
    }

    false
}

pub fn usable(path: &str) -> bool {
    let unprintable: Regex = Regex::new(r"(\p{Cc}|\p{Cf})").unwrap();

    !path.is_empty()
        && !path.contains("{{")
        && !path.starts_with("./")
        && !path.starts_with("../")
        && !too_broad(path)
        && !unprintable.is_match(path)
}