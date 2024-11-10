use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;
use crate::remote::RemoteAccessError;
use crate::DB;
use md5::{Context, Digest};
use reqwest::blocking::Response;

use std::io::Read;
use std::sync::atomic::AtomicU64;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, ErrorKind, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::Arc,
};
use urlencoding::encode;

use super::download_agent::GameDownloadError;
use super::download_thread_control_flag::DownloadThreadControl;

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

pub struct DropDownloadPipeline<R: Read, W: Write> {
    pub source: R,
    pub destination: DropWriter<W>,
    pub control_flag: DownloadThreadControl,
    pub progress: Arc<AtomicU64>,
    pub size: usize,
}
impl DropDownloadPipeline<Response, File> {
    fn new(
        source: Response,
        destination: DropWriter<File>,
        control_flag: DownloadThreadControl,
        progress: Arc<AtomicU64>,
        size: usize,
    ) -> Self {
        return Self {
            source,
            destination,
            control_flag,
            progress,
            size,
        };
    }

    fn copy(&mut self) -> Result<bool, io::Error> {
        let copy_buf_size = 512;
        let mut copy_buf = vec![0; copy_buf_size];
        let mut buf_writer = BufWriter::with_capacity(1024 * 1024, &mut self.destination);

        let mut current_size = 0;
        loop {
            if self.control_flag.get() == false {
                return Ok(false);
            }

            let bytes_read = self.source.read(&mut copy_buf)?;
            current_size += bytes_read;

            buf_writer.write_all(&copy_buf[0..bytes_read])?;
            self.progress.fetch_add(
                bytes_read.try_into().unwrap(),
                std::sync::atomic::Ordering::Relaxed,
            );

            if current_size == self.size {
                break;
            }
        }

        Ok(true)
    }

    fn finish(self) -> Result<Digest, io::Error> {
        let checksum = self.destination.finish()?;
        return Ok(checksum);
    }
}

pub fn download_game_chunk(
    ctx: DropDownloadContext,
    control_flag: DownloadThreadControl,
    progress: Arc<AtomicU64>,
) -> Result<bool, GameDownloadError> {
    // If we're paused
    if control_flag.get() {
        return Ok(false);
    }

    let base_url = DB.fetch_base_url();

    let client = reqwest::blocking::Client::new();
    let chunk_url = base_url
        .join(&format!(
            "/api/v1/client/chunk?id={}&version={}&name={}&chunk={}",
            // Encode the parts we don't trust
            ctx.game_id,
            encode(&ctx.version),
            encode(&ctx.file_name),
            ctx.index
        ))
        .unwrap();

    let header = generate_authorization_header();

    let response = client
        .get(chunk_url)
        .header("Authorization", header)
        .send()
        .map_err(|e| GameDownloadError::CommunicationError(RemoteAccessError::FetchError(e)))?;

    let mut destination = DropWriter::new(ctx.path);

    if ctx.offset != 0 {
        destination
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    let content_length = response.content_length();
    if content_length.is_none() {
        return Err(GameDownloadError::CommunicationError(
            RemoteAccessError::GenericErrror(
                "Invalid download endpoint, missing Content-Length header.".to_owned(),
            ),
        ));
    }

    let mut pipeline = DropDownloadPipeline::new(
        response,
        destination,
        control_flag,
        progress,
        content_length.unwrap().try_into().unwrap(),
    );

    let completed = pipeline.copy().unwrap();
    if !completed {
        return Ok(false);
    };

    let checksum = pipeline.finish().unwrap();

    let res = hex::encode(checksum.0);
    if res != ctx.checksum {
        return Err(GameDownloadError::ChecksumError);
    }

    return Ok(true);
}
