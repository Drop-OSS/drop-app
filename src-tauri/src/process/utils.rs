use std::path::PathBuf;

use futures_lite::io;
use sysinfo::{Disk, DiskRefreshKind, Disks};

use crate::error::application_download_error::ApplicationDownloadError;

pub fn get_disk_available(mount_point: PathBuf) -> Result<u64, ApplicationDownloadError> {
    let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage());

    let mut disk_iter = disks.into_iter().collect::<Vec<&Disk>>();
    disk_iter.sort_by(|a, b| {
        b.mount_point()
            .to_string_lossy()
            .len()
            .cmp(&a.mount_point().to_string_lossy().len())
    });

    for disk in disk_iter {
        if mount_point.starts_with(disk.mount_point()) {
            return Ok(disk.available_space());
        }
    }
    Err(ApplicationDownloadError::IoError(io::Error::other(
        "could not find disk of path",
    ).kind()))
}
