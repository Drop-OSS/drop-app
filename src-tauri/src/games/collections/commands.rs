use reqwest::Client;
use serde_json::json;
use url::Url;

use crate::{
    DB,
    database::db::DatabaseImpls,
    error::remote_access_error::RemoteAccessError,
    remote::{auth::generate_authorization_header, requests::make_request},
};

use super::collection::{Collection, Collections};

#[tauri::command]
pub async fn fetch_collections() -> Result<Collections, RemoteAccessError> {
    let client = Client::new();
    let response = make_request(&client, &["/api/v1/client/collection"], &[], async |r| {
        r.header("Authorization", generate_authorization_header().await)
    })
    .await?
    .send()
    .await?;

    Ok(response.json().await?)
}

#[tauri::command]
pub async fn fetch_collection(collection_id: String) -> Result<Collection, RemoteAccessError> {
    let client = Client::new();
    let response = make_request(
        &client,
        &["/api/v1/client/collection/", &collection_id],
        &[],
        async |r| r.header("Authorization", generate_authorization_header().await),
    )
    .await?
    .send()
    .await?;

    Ok(response.json().await?)
}

#[tauri::command]
pub async fn create_collection(name: String) -> Result<Collection, RemoteAccessError> {
    let client = Client::new();
    let base_url = DB.fetch_base_url().await;

    let base_url = Url::parse(&format!("{base_url}api/v1/client/collection/"))?;

    let response = client
        .post(base_url)
        .header("Authorization", generate_authorization_header().await)
        .json(&json!({"name": name}))
        .send()
        .await?;

    Ok(response.json().await?)
}

#[tauri::command]
pub async fn add_game_to_collection(
    collection_id: String,
    game_id: String,
) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let url = Url::parse(&format!(
        "{}api/v1/client/collection/{}/entry/",
        DB.fetch_base_url().await,
        collection_id
    ))?;

    client
        .post(url)
        .header("Authorization", generate_authorization_header().await)
        .json(&json!({"id": game_id}))
        .send()
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_collection(collection_id: String) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let base_url = Url::parse(&format!(
        "{}api/v1/client/collection/{}",
        DB.fetch_base_url().await,
        collection_id
    ))?;

    client
        .delete(base_url)
        .header("Authorization", generate_authorization_header().await)
        .send().await?;

    Ok(())
}
#[tauri::command]
pub async fn delete_game_in_collection(
    collection_id: String,
    game_id: String,
) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let base_url = Url::parse(&format!(
        "{}api/v1/client/collection/{}/entry",
        DB.fetch_base_url().await,
        collection_id
    ))?;

    client
        .delete(base_url)
        .header("Authorization", generate_authorization_header().await)
        .json(&json!({"id": game_id}))
        .send().await?;

    Ok(())
}
