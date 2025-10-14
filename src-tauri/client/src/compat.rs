use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Command, Stdio},
    sync::LazyLock,
};

use log::info;

pub static COMPAT_INFO: LazyLock<Option<CompatInfo>> = LazyLock::new(create_new_compat_info);

pub static UMU_LAUNCHER_EXECUTABLE: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let x = get_umu_executable();
    info!("{:?}", &x);
    x
});

#[derive(Clone)]
pub struct CompatInfo {
    pub umu_installed: bool,
}

fn create_new_compat_info() -> Option<CompatInfo> {
    #[cfg(target_os = "windows")]
    return None;

    let has_umu_installed = UMU_LAUNCHER_EXECUTABLE.is_some();
    Some(CompatInfo {
        umu_installed: has_umu_installed,
    })
}

const UMU_BASE_LAUNCHER_EXECUTABLE: &str = "umu-run";
const UMU_INSTALL_DIRS: [&str; 4] = ["/app/share", "/use/local/share", "/usr/share", "/opt"];

fn get_umu_executable() -> Option<PathBuf> {
    if check_executable_exists(UMU_BASE_LAUNCHER_EXECUTABLE) {
        return Some(PathBuf::from(UMU_BASE_LAUNCHER_EXECUTABLE));
    }

    for dir in UMU_INSTALL_DIRS {
        let p = PathBuf::from(dir).join(UMU_BASE_LAUNCHER_EXECUTABLE);
        if check_executable_exists(&p) {
            return Some(p);
        }
    }
    None
}
fn check_executable_exists<P: AsRef<OsStr>>(exec: P) -> bool {
    let has_umu_installed = Command::new(exec).stdout(Stdio::null()).output();
    has_umu_installed.is_ok()
}
