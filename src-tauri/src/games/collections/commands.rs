use std::sync::Mutex;

use reqwest::blocking::Client;

use crate::{error::remote_access_error::RemoteAccessError, games::{collections::collection::CollectionObject, library::Game}, remote::{auth::generate_authorization_header, requests::make_request}, AppState};

use super::collection::{Collection, Collections};

#[tauri::command]
pub fn fetch_collections() -> Result<Collections, RemoteAccessError> {
    println!("Fetching collection");
    let client = Client::new();
    let response = make_request(&client, &["/api/v1/client/collection"], &[], |r| {
        r.header("Authorization", generate_authorization_header())
    })?
    .send()?;

    let res = response.json().unwrap();

    return Ok(res)
}