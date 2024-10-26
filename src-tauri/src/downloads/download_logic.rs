use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;
use crate::DB;
use gxhash::{gxhash128, GxHasher};
use log::info;
use std::{fs::{File, OpenOptions}, hash::Hasher, io::{BufWriter, Seek, SeekFrom, Write}, path::PathBuf};
use urlencoding::encode;

pub struct FileWriter {
    file: File,
    hasher: GxHasher
}
impl FileWriter {
    fn new(path: PathBuf) -> Self {
        Self {
            file: OpenOptions::new().write(true).open(path).unwrap(),
            hasher: GxHasher::with_seed(0)
        }
    }
    fn finish(mut self) -> u128 {
        self.flush();
        self.hasher.finish_u128()
    }
}
impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.write(buf);
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
impl Seek for FileWriter {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}
pub fn download_game_chunk(ctx: DropDownloadContext) {
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

    let mut file: FileWriter = FileWriter::new(ctx.path);

    if ctx.offset != 0 {
        file
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    // let mut stream = BufWriter::with_capacity(1024, file);

    // Writing directly to disk to avoid write spikes that delay everything

    let data = response.bytes().unwrap();
    let checksum = gxhash128(&*data.clone(), 0);
    let res = file.write(&*data).unwrap();

    //response.copy_to(&mut file).unwrap();
    let res = hex::encode(file.finish().to_le_bytes());
    if res == ctx.checksum {
        info!("Matched Checksum {}", res);
    }
    else {
        info!("Checksum failed. Original: {}, Calculated: {}", ctx.checksum, res);
        info!("Other Checksum: {}", hex::encode(checksum.to_le_bytes()));
    }

    // stream.flush().unwrap();
}
