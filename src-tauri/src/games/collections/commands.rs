use serde_json::json;

use crate::{
    error::remote_access_error::RemoteAccessError,
    remote::{
        auth::generate_authorization_header,
        requests::{generate_url, make_authenticated_get},
        utils::DROP_CLIENT_ASYNC,
    },
};

use super::collection::{Collection, Collections};

#[tauri::command]
pub async fn fetch_collections() -> Result<Collections, RemoteAccessError> {
    let response =
        make_authenticated_get(generate_url(&["/api/v1/client/collection"], &[])?).await?;

    Ok(response.json().await?)
}

#[tauri::command]
pub async fn fetch_collection(collection_id: String) -> Result<Collection, RemoteAccessError> {
    let response = make_authenticated_get(generate_url(
        &["/api/v1/client/collection/", &collection_id],
        &[],
    )?)
    .await?;

    Ok(response.json().await?)
}

#[tauri::command]
pub async fn create_collection(name: String) -> Result<Collection, RemoteAccessError> {
    let client = DROP_CLIENT_ASYNC.clone();
    let url = generate_url(&["/api/v1/client/collection"], &[])?;

    let response = client
        .post(url)
        .header("Authorization", generate_authorization_header())
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
    let client = DROP_CLIENT_ASYNC.clone();

    let url = generate_url(&["/api/v1/client/collection", &collection_id, "entry"], &[])?;

    client
        .post(url)
        .header("Authorization", generate_authorization_header())
        .json(&json!({"id": game_id}))
        .send()
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_collection(collection_id: String) -> Result<bool, RemoteAccessError> {
    let client = DROP_CLIENT_ASYNC.clone();

    let url = generate_url(&["/api/v1/client/collection", &collection_id], &[])?;

    let response = client
        .delete(url)
        .header("Authorization", generate_authorization_header())
        .send()
        .await?;

    Ok(response.json().await?)
}
#[tauri::command]
pub async fn delete_game_in_collection(
    collection_id: String,
    game_id: String,
) -> Result<(), RemoteAccessError> {
    let client = DROP_CLIENT_ASYNC.clone();

    let url = generate_url(&["/api/v1/client/collection", &collection_id, "entry"], &[])?;

    client
        .delete(url)
        .header("Authorization", generate_authorization_header())
        .json(&json!({"id": game_id}))
        .send().await?;

    Ok(())
}
