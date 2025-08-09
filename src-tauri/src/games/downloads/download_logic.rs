use crate::download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use crate::download_manager::util::progress_object::ProgressHandle;
use crate::error::application_download_error::ApplicationDownloadError;
use crate::error::drop_server_error::DropServerError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::downloads::manifest::{
    ChunkBody, DownloadBucket, DownloadContext, DownloadDrop, DropValidateContext,
};
use crate::remote::auth::generate_authorization_header;
use crate::remote::requests::generate_url;
use crate::remote::utils::DROP_CLIENT_SYNC;
use http::response;
use log::{debug, info, warn};
use md5::{Context, Digest};
use reqwest::blocking::{RequestBuilder, Response};
use serde_json::de;
use tokio::net::unix::pipe;

use std::fs::{Permissions, set_permissions};
use std::io::Read;
use std::ops::Sub;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Instant;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};

static MAX_PACKET_LENGTH: usize = 4096 * 4;

pub struct DropWriter<W: Write> {
    hasher: Context,
    destination: BufWriter<W>,
    progress: ProgressHandle,
}
impl DropWriter<File> {
    fn new(path: PathBuf, progress: ProgressHandle) -> Result<Self, io::Error> {
        let destination = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        Ok(Self {
            destination: BufWriter::with_capacity(1 * 1024 * 1024, destination),
            hasher: Context::new(),
            progress,
        })
    }

    fn finish(mut self) -> io::Result<Digest> {
        self.flush()?;
        Ok(self.hasher.compute())
    }
}
// Write automatically pushes to file and hasher
impl Write for DropWriter<File> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hasher
            .write_all(buf)
            .map_err(|e| io::Error::other(format!("Unable to write to hasher: {e}")))?;
        let bytes_written = self.destination.write(buf)?;
        self.progress.add(bytes_written);

        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.hasher.flush()?;
        self.destination.flush()
    }
}
// Seek moves around destination output
impl Seek for DropWriter<File> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.destination.seek(pos)
    }
}

pub struct DropDownloadPipeline<'a, R: Read, W: Write> {
    pub source: R,
    pub drops: Vec<DownloadDrop>,
    pub destination: Vec<DropWriter<W>>,
    pub control_flag: &'a DownloadThreadControl,
}

impl<'a> DropDownloadPipeline<'a, Response, File> {
    fn new(
        source: Response,
        drops: Vec<DownloadDrop>,
        control_flag: &'a DownloadThreadControl,
        progress: ProgressHandle,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            source,
            destination: drops
                .iter()
                .map(|drop| DropWriter::new(drop.path.clone(), progress.clone()))
                .try_collect()?,
            drops,
            control_flag,
        })
    }

    fn copy(&mut self) -> Result<bool, io::Error> {
        let mut copy_buffer = [0u8; MAX_PACKET_LENGTH];
        for (index, drop) in self.drops.iter().enumerate() {
            let destination = self
                .destination
                .get_mut(index)
                .ok_or(io::Error::other("no destination"))
                .unwrap();
            let mut remaining = drop.length;
            if drop.start != 0 {
                destination.seek(SeekFrom::Start(drop.start.try_into().unwrap()))?;
            }
            loop {
                let size = MAX_PACKET_LENGTH.min(remaining);
                self.source.read_exact(&mut copy_buffer[0..size])?;
                remaining -= size;

                destination.write_all(&copy_buffer[0..size])?;

                if remaining == 0 {
                    break;
                };
            }
        }

        Ok(true)
    }

    fn finish(self) -> Result<Vec<Digest>, io::Error> {
        let checksums = self
            .destination
            .into_iter()
            .map(|e| e.finish())
            .try_collect()?;
        Ok(checksums)
    }
}

pub fn download_game_bucket(
    bucket: &DownloadBucket,
    ctx: &DownloadContext,
    control_flag: &DownloadThreadControl,
    progress: ProgressHandle,
) -> Result<bool, ApplicationDownloadError> {
    // If we're paused
    if control_flag.get() == DownloadThreadControlFlag::Stop {
        progress.set(0);
        return Ok(false);
    }

    let header = generate_authorization_header();

    let url = generate_url(&["/api/v2/client/chunk"], &[])
        .map_err(ApplicationDownloadError::Communication)?;

    let body = ChunkBody::create(ctx, &bucket.drops);

    let response = DROP_CLIENT_SYNC
        .post(url)
        .json(&body)
        .header("Authorization", header)
        .send()
        .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

    if response.status() != 200 {
        info!("chunk request got status code: {}", response.status());
        let raw_res = response.text().map_err(|e| {
            ApplicationDownloadError::Communication(RemoteAccessError::FetchError(e.into()))
        })?;
        info!("{}", raw_res);
        if let Ok(err) = serde_json::from_str::<DropServerError>(&raw_res) {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::InvalidResponse(err),
            ));
        }
        return Err(ApplicationDownloadError::Communication(
            RemoteAccessError::UnparseableResponse(raw_res),
        ));
    }

    let lengths = response
        .headers()
        .get("Content-Lengths")
        .ok_or(ApplicationDownloadError::Communication(
            RemoteAccessError::UnparseableResponse("missing Content-Lengths header".to_owned()),
        ))?
        .to_str()
        .unwrap()
        .split(",");

    for (i, raw_length) in lengths.enumerate() {
        let length = raw_length.parse::<usize>().unwrap_or(0);
        let drop = bucket.drops.get(i).unwrap();
        if drop.length == length {
        } else {
            warn!(
                "for {}, expected {}, got {} ({})",
                drop.filename, drop.length, raw_length, length
            );
            return Err(ApplicationDownloadError::DownloadError);
        }
    }

    let mut pipeline =
        DropDownloadPipeline::new(response, bucket.drops.clone(), control_flag, progress)
            .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;

    let completed = pipeline
        .copy()
        .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;
    if !completed {
        return Ok(false);
    }

    // If we complete the file, set the permissions (if on Linux)
    #[cfg(unix)]
    {
        for drop in bucket.drops.iter() {
            let permissions = Permissions::from_mode(drop.permissions);
            set_permissions(drop.path.clone(), permissions)
                .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;
        }
    }

    let checksums = pipeline
        .finish()
        .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;

    for (index, drop) in bucket.drops.iter().enumerate() {
        let res = hex::encode(**checksums.get(index).unwrap());
        if res != drop.checksum {
            return Err(ApplicationDownloadError::Checksum);
        }
    }

    Ok(true)
}
