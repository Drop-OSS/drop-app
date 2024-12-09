use crate::{auth::generate_authorization_header, db::DatabaseImpls, remote::RemoteAccessError, DB};


pub async fn register() -> Result<String, RemoteAccessError> {
    let base_url = DB.fetch_base_url();
    let registration_url = base_url.join("/api/v1/client/capability").unwrap();
    let header = generate_authorization_header();


    let client = reqwest::blocking::Client::new();
    client
        .post(registration_url)
        .header("Authorization", header)
        .send()?;

    return Ok(String::new())
}