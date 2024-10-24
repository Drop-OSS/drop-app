use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
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

    let index = ctx.index;
    let chunk = ctx.file_chunk;

    let client = reqwest::blocking::Client::new();
    let chunk_url = base_url.join(
        &format!(
            "/api/v1/client/chunk?id={}&version={}&name={}&chunk={}",
            ctx.game_id,
            ctx.version,
            ctx.file_name,
            index
        )).unwrap();

    let header = generate_authorization_header();

    let response = client
        .get(chunk_url)
        .header("Authorization", header)
        .send()
        .unwrap();
    let response_data = response.bytes().unwrap();
    
    
    write_to_file(ctx.file, CHUNK_SIZE * index as u64, response_data.to_vec());
    // Need to implement actual download logic
}

fn write_to_file(file: Arc<Mutex<File>>, offset: u64, data: Vec<u8>) {
    let mut lock = file.lock().unwrap();
    
    if offset != 0 {
        lock.seek(SeekFrom::Start(offset)).expect("Failed to seek to file offset");
    }
    
    lock.write_all(&data).unwrap();
}