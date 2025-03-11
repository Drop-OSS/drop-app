use reqwest::blocking::Client;
use url::Url;

use crate::{database::db::DatabaseImpls, error::remote_access_error::RemoteAccessError, remote::{auth::generate_authorization_header, requests::make_request}, DB};

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
pub fn fetch_collection(id: String) -> Result<Collection, RemoteAccessError> {
    let client = Client::new();
    let response = make_request(&client, &["/api/v1/client/collection/", &id], &[], |r| {
        r.header("Authorization", generate_authorization_header())
    })?
    .send()?;

    Ok(response.json()?)
}

#[tauri::command]
pub fn create_collection(name: String) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let base_url = DB.fetch_base_url();

    let base_url = Url::parse(&format!("{}api/v1/client/collection/", base_url))?;

    let response = client
        .post(base_url)
        .header("Authorization", generate_authorization_header())
        .json(&{name});

    println!("{:?}", response);
    

    println!("{}", response.send()?.text().unwrap());


    Ok(())
}

#[tauri::command]
pub fn add_game_to_collection(name: String) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let url = Url::parse(&format!("{}api/v1/client/collection/{}/entry/", DB.fetch_base_url(), name))?;

    let response = client
        .post(url)
        .header("Authorization", generate_authorization_header())
        .send()?;
    Ok(())
}

#[tauri::command]
pub fn delete_collection(id: String) -> Result<bool, RemoteAccessError> {
    let client = Client::new();
    let base_url = Url::parse(&format!("{}api/v1/client/collection/{}", DB.fetch_base_url(), id))?;

    let response = client
        .delete(base_url)
        .header("Authorization", generate_authorization_header())
        .send()?;

    Ok(response.json()?)
}
#[tauri::command]
pub fn delete_game_in_collection(id: String) -> Result<(), RemoteAccessError> {
    let client = Client::new();
    let base_url = Url::parse(&format!("{}api/v1/client/collection/{}/entry", DB.fetch_base_url(), id))?;

    client
        .delete(base_url)
        .header("Authorization", generate_authorization_header())
        .json(&{id})
        .send()?;

    Ok(())
}