use reqwest::blocking::Client;
use serde_json::json;
use url::Url;

use crate::{
    database::db::DatabaseImpls,
    error::remote_access_error::RemoteAccessError,
    remote::{auth::generate_authorization_header, requests::make_request},
    DB,
};

use super::collection::{Collection, Collections};

#[tauri::command]
pub fn fetch_collections() -> Result<Collections, RemoteAccessError> {
    let client = Client::new();
    let response = make_request(&client, &["/api/v1/client/collection"], &[], |r| {
        r.header("Authorization", generate_authorization_header())
    })?
    .send()?;

    Ok(response.json()?)
}

#[tauri::command]
pub fn fetch_collection(collection_id: String) -> Result<Collection, RemoteAccessError> {
    let client = Client::new();
    let response = make_request(
        &client,
        &["/api/v1/client/collection/", &collection_id],
        &[],
        |r| r.header("Authorization", generate_authorization_header()),
    )?
    .send()?;

    Ok(response.json()?)
}

#[tauri::command]
pub fn create_collection(name: String) -> Result<Collection, RemoteAccessError> {
    let client = Client::new();
    let base_url = DB.fetch_base_url();

    let base_url = Url::parse(&format!("{base_url}api/v1/client/collection/"))?;

    let response = client
        .post(base_url)
        .header("Authorization", generate_authorization_header())
        .json(&json!({"name": name}))
        .send()?;

    Ok(response.json()?)
}

#[tauri::command]
pub fn add_game_to_collection(
    collection_id: String,
    game_id: String,
) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let url = Url::parse(&format!(
        "{}api/v1/client/collection/{}/entry/",
        DB.fetch_base_url(),
        collection_id
    ))?;

    client
        .post(url)
        .header("Authorization", generate_authorization_header())
        .json(&json!({"id": game_id}))
        .send()?;
    Ok(())
}

#[tauri::command]
pub fn delete_collection(collection_id: String) -> Result<bool, RemoteAccessError> {
    let client = Client::new();
    let base_url = Url::parse(&format!(
        "{}api/v1/client/collection/{}",
        DB.fetch_base_url(),
        collection_id
    ))?;

    let response = client
        .delete(base_url)
        .header("Authorization", generate_authorization_header())
        .send()?;

    Ok(response.json()?)
}
#[tauri::command]
pub fn delete_game_in_collection(
    collection_id: String,
    game_id: String,
) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let base_url = Url::parse(&format!(
        "{}api/v1/client/collection/{}/entry",
        DB.fetch_base_url(),
        collection_id
    ))?;

    client
        .delete(base_url)
        .header("Authorization", generate_authorization_header())
        .json(&json!({"id": game_id}))
        .send()?;

    Ok(())
}
