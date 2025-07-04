use crate::download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use crate::download_manager::util::progress_object::ProgressHandle;
use crate::error::application_download_error::ApplicationDownloadError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::downloads::drop_data::DropData;
use crate::games::downloads::manifest::{DropDownloadContext, DropManifest};
use crate::remote::auth::generate_authorization_header;
use crate::remote::requests::make_request;
use log::{debug, warn};
use md5::{Context, Digest};
use native_model::Decode;
use reqwest::blocking::{RequestBuilder, Response};
use reqwest::Client;

use std::fs::{set_permissions, Permissions};
use std::io::{copy, ErrorKind, Read};
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::thread::sleep;
use std::time::Duration;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};

pub struct DropWriter<W: Write> {
    hasher: Context,
    destination: W,
}
impl DropWriter<File> {
    fn new(path: PathBuf) -> Self {
        Self {
            destination: OpenOptions::new().write(true).open(path).unwrap(),
            hasher: Context::new(),
        }
    }

    fn finish(mut self) -> io::Result<Digest> {
        self.flush().unwrap();
        Ok(self.hasher.compute())
    }
}
// Write automatically pushes to file and hasher
impl Write for DropWriter<File> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hasher.write_all(buf).map_err(|e| {
            io::Error::new(
                ErrorKind::Other,
                format!("Unable to write to hasher: {}", e),
            )
        })?;
        self.destination.write(buf)
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
    pub progress: ProgressHandle,
    pub size: usize,
}
impl<'a> DropDownloadPipeline<'a, Response, File> {
    fn new(
        source: Response,
        destination: DropWriter<File>,
        control_flag: &'a DownloadThreadControl,
        progress: ProgressHandle,
        size: usize,
    ) -> Self {
        Self {
            source,
            destination,
            control_flag,
            progress,
            size,
        }
    }

    fn copy(&mut self) -> Result<bool, io::Error> {
        let copy_buf_size = 512;
        let mut copy_buf = vec![0; copy_buf_size];
        let mut buf_writer = BufWriter::with_capacity(1024 * 1024, &mut self.destination);

        let mut current_size = 0;
        loop {
            if self.control_flag.get() == DownloadThreadControlFlag::Stop {
                buf_writer.flush()?;
                return Ok(false);
            }

            let bytes_read = self.source.read(&mut copy_buf)?;
            current_size += bytes_read;

            buf_writer.write_all(&copy_buf[0..bytes_read])?;
            self.progress.add(bytes_read);

            if current_size == self.size {
                break;
            }
        }
        buf_writer.flush()?;

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
    debug!(
        "Starting download chunk {}, {}, {} #{}",
        ctx.file_name, ctx.index, ctx.offset, ctx.checksum
    );
    // If we're paused
    if control_flag.get() == DownloadThreadControlFlag::Stop {
        progress.set(0);
        return Ok(false);
    }

    let response = request
        .send()
        .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

    if response.status() != 200 {
        let err = response.json().unwrap();
        return Err(ApplicationDownloadError::Communication(
            RemoteAccessError::InvalidResponse(err),
        ));
    }

    let mut destination = DropWriter::new(ctx.path.clone());

    if ctx.offset != 0 {
        destination
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    let content_length = response.content_length();
    if content_length.is_none() {
        warn!("recieved 0 length content from server");
        return Err(ApplicationDownloadError::Communication(
            RemoteAccessError::InvalidResponse(response.json().unwrap()),
        ));
    }

    let length = content_length.unwrap().try_into().unwrap();

    if length != ctx.length {
        return Err(ApplicationDownloadError::DownloadError);
    }

    let mut pipeline =
        DropDownloadPipeline::new(response, destination, control_flag, progress, length);

    let completed = pipeline
        .copy()
        .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;
    if !completed {
        return Ok(false);
    };

    // If we complete the file, set the permissions (if on Linux)
    #[cfg(unix)]
    {
        let permissions = Permissions::from_mode(ctx.permissions);
        set_permissions(ctx.path.clone(), permissions).unwrap();
    }

    let checksum = pipeline
        .finish()
        .map_err(|e| ApplicationDownloadError::IoError(e.kind()))?;

    let res = hex::encode(checksum.0);
    if res != ctx.checksum {
        return Err(ApplicationDownloadError::Checksum);
    }

    debug!(
        "Successfully finished download #{}, copied {} bytes",
        ctx.checksum, length
    );

    Ok(true)
}