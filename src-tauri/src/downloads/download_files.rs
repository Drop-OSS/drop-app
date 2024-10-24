use log::info;
use crate::auth::generate_authorization_header;
use crate::DB;
use crate::db::DatabaseImpls;
use crate::downloads::manifest::DropDownloadContext;


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
    println!("Response text: {}", response.text().unwrap());
    // Need to implement actual download logic
}