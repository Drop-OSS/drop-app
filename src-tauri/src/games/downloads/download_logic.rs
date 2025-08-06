use crate::download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use crate::download_manager::util::progress_object::ProgressHandle;
use crate::error::application_download_error::ApplicationDownloadError;
use crate::error::drop_server_error::DropServerError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::downloads::manifest::DropDownloadContext;
use crate::remote::auth::generate_authorization_header;
use http::response;
use log::{debug, info, warn};
use md5::{Context, Digest};
use reqwest::blocking::{RequestBuilder, Response};

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
    pub destination: DropWriter<W>,
    pub control_flag: &'a DownloadThreadControl,
    pub size: usize,
}

impl<'a> Seek for DropDownloadPipeline<'a, Response, File> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.destination.seek(pos)
    }
}

impl<'a> DropDownloadPipeline<'a, Response, File> {
    fn new(
        source: Response,
        destination: PathBuf,
        control_flag: &'a DownloadThreadControl,
        progress: ProgressHandle,
        size: usize,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            source,
            destination: DropWriter::new(destination, progress)?,
            control_flag,
            size,
        })
    }

    fn copy(&mut self) -> Result<bool, io::Error> {
        io::copy(&mut self.source, &mut self.destination)?;

        Ok(true)
    }

    fn finish(self) -> Result<Digest, io::Error> {
        let checksum = self.destination.finish()?;
        Ok(checksum)
    }
}

pub fn download_game_chunk(
    ctx: &DropDownloadContext,
    control_flag: &DownloadThreadControl,
    progress: ProgressHandle,
    request: RequestBuilder,
) -> Result<bool, ApplicationDownloadError> {
    // If we're paused
    if control_flag.get() == DownloadThreadControlFlag::Stop {
        progress.set(0);
        return Ok(false);
    }

    let start = Instant::now();

    debug!("started chunk {}", ctx.checksum);

    let header = generate_authorization_header();
    let header_time = start.elapsed();

    let response = request
        .header("Authorization", header)
        .send()
        .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

    let response_time = start.elapsed();

    if response.status() != 200 {
        debug!("chunk request got status code: {}", response.status());
        let raw_res = response.text().map_err(|e| {
            ApplicationDownloadError::Communication(RemoteAccessError::FetchError(e.into()))
        })?;
        if let Ok(err) = serde_json::from_str::<DropServerError>(&raw_res) {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::InvalidResponse(err),
            ));
        }
        return Err(ApplicationDownloadError::Communication(
            RemoteAccessError::UnparseableResponse(raw_res),
        ));
    }

    let length = response
        .content_length()
        .ok_or(ApplicationDownloadError::Communication(
            RemoteAccessError::UnparseableResponse("missing Content-Length header".to_owned()),
        ))?
        .try_into()
        .unwrap();

    if length != ctx.length {
        return Err(ApplicationDownloadError::DownloadError);
    }

    let pipeline_start = start.elapsed();

    let mut pipeline =
        DropDownloadPipeline::new(response, ctx.path.clone(), control_flag, progress, length)
            .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;

    if ctx.offset != 0 {
        pipeline
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    let pipeline_setup = start.elapsed();

    let completed = pipeline
        .copy()
        .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;
    if !completed {
        return Ok(false);
    }

    let pipeline_finish = start.elapsed();

    // If we complete the file, set the permissions (if on Linux)
    #[cfg(unix)]
    {
        let permissions = Permissions::from_mode(ctx.permissions);
        set_permissions(ctx.path.clone(), permissions)
            .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;
    }

    let checksum = pipeline
        .finish()
        .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;

    let checksum_finish = start.elapsed();

    let res = hex::encode(checksum.0);
    if res != ctx.checksum {
        return Err(ApplicationDownloadError::Checksum);
    }

    let header_update = header_time.as_millis();
    let response_update = response_time.sub(header_time).as_millis();
    let pipeline_start_update = pipeline_start.sub(response_time).as_millis();
    let pipeline_setup_update = pipeline_setup.sub(pipeline_start).as_millis();
    let pipeline_finish_update = pipeline_finish.sub(pipeline_setup).as_millis();
    let checksum_update = checksum_finish.sub(pipeline_finish).as_millis();

    debug!(
        "\nheader: {}\nresponse: {}\npipeline start: {}\npipeline setup: {}\npipeline finish: {}\nchecksum finish: {}",
        header_update,
        response_update,
        pipeline_start_update,
        pipeline_setup_update,
        pipeline_finish_update,
        checksum_update
    );

    debug!("finished chunk {}", ctx.checksum);

    Ok(true)
}
