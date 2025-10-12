use games::collections::collection::{Collection, Collections};
use remote::{
    auth::generate_authorization_header,
    cache::{cache_object, get_cached_object},
    error::RemoteAccessError,
    requests::{generate_url, make_authenticated_get},
    utils::DROP_CLIENT_ASYNC,
};
use serde_json::json;

#[tauri::command]
pub async fn fetch_collections(
    hard_refresh: Option<bool>,
) -> Result<Collections, RemoteAccessError> {
    let do_hard_refresh = hard_refresh.unwrap_or(false);
    if !do_hard_refresh && let Ok(cached_response) = get_cached_object::<Collections>("collections")
    {
        return Ok(cached_response);
    }

    let response =
        make_authenticated_get(generate_url(&["/api/v1/client/collection"], &[])?).await?;

    let collections: Collections = response.json().await?;

    cache_object("collections", &collections)?;

    Ok(collections)
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
        .send()
        .await?;

    Ok(())
}
