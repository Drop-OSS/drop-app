use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::sync::{Arc, Mutex};
use log::info;
use uuid::Bytes;
use crate::auth::generate_authorization_header;
use crate::DB;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;

const CHUNK_SIZE: u64 = 1024 * 1024 * 64;
pub fn download_game_chunk(ctx: DropDownloadContext) {
    info!("Downloading game chunk");
    let base_url = DB.fetch_base_url();


    let client = reqwest::blocking::Client::new();
    let chunk_url = base_url.join(
        &format!(
            "/api/v1/client/chunk?id={}&version={}&name={}&chunk={}",
            ctx.game_id,
            ctx.version,
            ctx.file_name,
            ctx.index
        )).unwrap();

    let header = generate_authorization_header();

    let response = client
        .get(chunk_url)
        .header("Authorization", header)
        .send()
        .unwrap();
    let response_data = response.bytes().unwrap();
    
    info!("Writing data to chunk at offset {}", CHUNK_SIZE * ctx.index as u64);
    write_to_file(ctx.file, ctx.index as u64, response_data.to_vec());
    // Need to implement actual download logic
}

fn write_to_file(file: Arc<Mutex<File>>, index: u64, data: Vec<u8>) {
    let mut lock = file.lock().unwrap();
    
    if index != 0 {
        lock.seek(SeekFrom::Start(index * CHUNK_SIZE)).expect("Failed to seek to file offset");
    }
    
    lock.write_all(&data).unwrap();
}