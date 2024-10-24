use crate::auth::generate_authorization_header;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;
use crate::DB;
use log::info;
use urlencoding::encode;
use std::io::{BufWriter, Seek, SeekFrom, Write};

pub fn download_game_chunk(ctx: DropDownloadContext) {
    let base_url = DB.fetch_base_url();

    let client = reqwest::blocking::Client::new();
    let chunk_url = base_url
        .join(&format!(
            "/api/v1/client/chunk?id={}&version={}&name={}&chunk={}",
            // Encode the parts we don't trust
            ctx.game_id, encode(&ctx.version), encode(&ctx.file_name), ctx.index
        ))
        .unwrap();

    let header = generate_authorization_header();

    let mut response = client
        .get(chunk_url)
        .header("Authorization", header)
        .send()
        .unwrap();

    let mut file_lock = ctx.file.lock().unwrap();

    if ctx.offset != 0 {
        file_lock
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    let mut stream = BufWriter::with_capacity(1024, file_lock.try_clone().unwrap());

    response.copy_to(&mut stream).unwrap();
}
