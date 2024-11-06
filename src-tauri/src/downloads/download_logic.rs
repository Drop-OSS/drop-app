use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;
use crate::DB;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use log::{error, info};
use md5::{Context, Digest};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Error, ErrorKind, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use urlencoding::encode;

pub struct DropFileWriter {
    file: File,
    hasher: Context,
    callback: Arc<AtomicBool>,
    progress: Arc<RelaxedCounter>,
}
impl DropFileWriter {
    fn new(path: PathBuf, callback: Arc<AtomicBool>, progress: Arc<RelaxedCounter>) -> Self {
        Self {
            file: OpenOptions::new().write(true).open(path).unwrap(),
            hasher: Context::new(),
            callback,
            progress,
        }
    }
    fn finish(mut self) -> io::Result<Digest> {
        self.flush().unwrap();
        Ok(self.hasher.compute())
    }
}
// TODO: Implement error handling
impl Write for DropFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.callback.load(Ordering::Acquire) {
            return Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Interrupt command recieved",
            ));
        }
        let len = buf.len();
        self.progress.add(len);

        //info!("Writing data to writer");
        self.hasher.write_all(buf).unwrap();
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.hasher.flush()?;
        self.file.flush()
    }
}
impl Seek for DropFileWriter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}
pub fn download_game_chunk(
    ctx: DropDownloadContext,
    callback: Arc<AtomicBool>,
    progress: Arc<RelaxedCounter>,
) {
    if callback.load(Ordering::Acquire) {
        info!("Callback stopped download at start");
        return;
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

    let mut response = client
        .get(chunk_url)
        .header("Authorization", header)
        .send()
        .unwrap();

    let mut file: DropFileWriter = DropFileWriter::new(ctx.path, callback, progress);

    if ctx.offset != 0 {
        file.seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    // Writing everything to disk directly is probably slightly faster in terms of disk
    // speed because it balances out the writes, but this is better than the performance 
    // loss from constantly reading the callbacks

    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    match io::copy(&mut response, &mut writer) {
        Ok(_) => {}
        Err(e) => {
            info!("Copy errored with error {}", e)
        }
    }
    writer.flush().unwrap();
    let file = match writer.into_inner() {
        Ok(file) => file,
        Err(_) => {
            error!("Failed to acquire writer from BufWriter");
            return;
        }
    };

    let res = hex::encode(file.finish().unwrap().0);
    if res != ctx.checksum {
        info!(
            "Checksum failed. Original: {}, Calculated: {} for {}",
            ctx.checksum, res, ctx.file_name
        );
    }

}
