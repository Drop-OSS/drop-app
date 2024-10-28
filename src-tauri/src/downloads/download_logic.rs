use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;
use crate::DB;
use gxhash::{gxhash128, GxHasher};
use log::info;
use md5::{Context, Digest};
use std::{fs::{File, OpenOptions}, hash::Hasher, io::{self, Seek, SeekFrom, Write}, path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use urlencoding::encode;

pub struct DropFileWriter {
    file: File,
    hasher: Context,
    callback: Arc<AtomicBool>
}
impl DropFileWriter {
    fn new(path: PathBuf, callback: Arc<AtomicBool>) -> Self {
        Self {
            file: OpenOptions::new().write(true).open(path).unwrap(),
            hasher: Context::new(),
            callback
        }
    }
    fn finish(mut self) -> io::Result<Digest> {
        self.flush().unwrap();
        Ok(self.hasher.compute())
    }
}
impl Write for DropFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.callback.load(Ordering::Acquire) {
            
        }
        self.hasher.write_all(buf).unwrap();
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.hasher.flush()?;
        self.file.flush()
    }
}
impl Seek for DropFileWriter {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}
pub fn download_game_chunk(ctx: DropDownloadContext, callback: Arc<AtomicBool>) {
    if callback.load(Ordering::Acquire) {
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

    let mut file: DropFileWriter = DropFileWriter::new(ctx.path);

    if ctx.offset != 0 {
        file
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    // let mut stream = BufWriter::with_capacity(1024, file);

    // Writing directly to disk to avoid write spikes that delay everything


    response.copy_to(&mut file).unwrap();
    let res = hex::encode(file.finish().unwrap().0);
    if res != ctx.checksum {  
        info!("Checksum failed. Original: {}, Calculated: {} for {}", ctx.checksum, res, ctx.file_name);
    }

    // stream.flush().unwrap();
}
