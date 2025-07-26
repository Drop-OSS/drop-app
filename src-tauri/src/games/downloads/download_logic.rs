use crate::download_manager::util::download_thread_control_flag::{
    DownloadThreadControl, DownloadThreadControlFlag,
};
use crate::download_manager::util::progress_object::ProgressHandle;
use crate::error::application_download_error::ApplicationDownloadError;
use crate::error::drop_server_error::DropServerError;
use crate::error::remote_access_error::RemoteAccessError;
use crate::games::downloads::manifest::DropDownloadContext;
use crate::remote::auth::generate_authorization_header;
use futures::TryStreamExt;
use log::{debug, warn};
use md5::{Context, Digest};
use reqwest::RequestBuilder;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tokio_util::io::StreamReader;

use std::fs::{Permissions, set_permissions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    io::{self, SeekFrom},
    path::PathBuf,
};

pub struct DropWriter<W: AsyncWrite> {
    hasher: Context,
    destination: W,
}
impl DropWriter<File> {
    async fn new(path: PathBuf) -> Self {
        Self {
            destination: OpenOptions::new().write(true).open(path).await.unwrap(),
            hasher: Context::new(),
        }
    }

    async fn finish(mut self) -> io::Result<Digest> {
        self.flush().await.unwrap();
        Ok(self.hasher.compute())
    }

    async fn write(&mut self, mut buf: &[u8]) -> io::Result<()> {
        self.hasher
            .write_all(buf)
            .map_err(|e| io::Error::other(format!("Unable to write to hasher: {e}")))?;
        self.destination.write_all_buf(&mut buf).await
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.hasher.flush()?;
        self.destination.flush().await
    }

    async fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.destination.seek(pos).await
    }
}

pub struct DropDownloadPipeline<'a, R: AsyncRead, W: AsyncWrite> {
    pub source: R,
    pub destination: DropWriter<W>,
    pub control_flag: &'a DownloadThreadControl,
    pub progress: ProgressHandle,
    pub size: usize,
}
impl<'a, R> DropDownloadPipeline<'a, R, File>
where
    R: AsyncRead + Unpin,
{
    fn new(
        source: R,
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

    async fn copy(&mut self) -> Result<bool, io::Error> {
        let copy_buf_size = 512;
        let mut copy_buf = vec![0; copy_buf_size];

        let mut current_size = 0;
        loop {
            if self.control_flag.get() == DownloadThreadControlFlag::Stop {
                self.destination.flush().await?;
                return Ok(false);
            }

            let mut bytes_read = self.source.read(&mut copy_buf).await?;
            current_size += bytes_read;

            if current_size > self.size {
                let over = current_size - self.size;
                warn!("server sent too many bytes... {over} over");
                bytes_read -= over;
                current_size = self.size;
            }

            self.destination.write(&copy_buf[0..bytes_read]).await?;
            self.progress.add(bytes_read);

            if current_size >= self.size {
                debug!(
                    "finished with final size of {} vs {}",
                    current_size, self.size
                );
                break;
            }
        }
        self.destination.flush().await?;

        Ok(true)
    }

    async fn finish(self) -> Result<Digest, io::Error> {
        let checksum = self.destination.finish().await?;
        Ok(checksum)
    }
}

pub async fn download_game_chunk(
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
    let response = request
        .header("Authorization", generate_authorization_header().await)
        .send()
        .await
        .map_err(|e| ApplicationDownloadError::Communication(e.into()))?;

    if response.status() != 200 {
        debug!("chunk request got status code: {}", response.status());
        let raw_res = response.text().await.unwrap();
        if let Ok(err) = serde_json::from_str::<DropServerError>(&raw_res) {
            return Err(ApplicationDownloadError::Communication(
                RemoteAccessError::InvalidResponse(err),
            ));
        };
        return Err(ApplicationDownloadError::Communication(
            RemoteAccessError::UnparseableResponse(raw_res),
        ));
    }

    let mut destination = DropWriter::new(ctx.path.clone()).await;

    if ctx.offset != 0 {
        destination
            .seek(SeekFrom::Start(ctx.offset))
            .await
            .expect("Failed to seek to file offset");
    }

    let content_length = response.content_length();
    if content_length.is_none() {
        warn!("recieved 0 length content from server");
        return Err(ApplicationDownloadError::Communication(
            RemoteAccessError::InvalidResponse(response.json().await.unwrap()),
        ));
    }

    let length = content_length.unwrap().try_into().unwrap();

    if length != ctx.length {
        return Err(ApplicationDownloadError::DownloadError);
    }

    let response_stream = StreamReader::new(
        response
            .bytes_stream()
            .map_err(|e| std::io::Error::other(e)),
    );
    let mut pipeline =
        DropDownloadPipeline::new(response_stream, destination, control_flag, progress, length);

    let completed = pipeline
        .copy()
        .await
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
        .await
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
