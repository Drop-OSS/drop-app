use std::{collections::HashMap, env, sync::Mutex};

use chrono::Utc;
use drop_database::{borrow_db_checked, borrow_db_mut_checked, models::data::DatabaseAuth, runtime_models::User};
use drop_errors::{drop_server_error::DropServerError, remote_access_error::RemoteAccessError};
use droplet_rs::ssl::sign_nonce;
use gethostname::gethostname;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{requests::make_authenticated_get, utils::{DROP_CLIENT_ASYNC, DROP_CLIENT_SYNC}};

use super::requests::generate_url;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityConfiguration {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitiateRequestBody {
    name: String,
    platform: String,
    capabilities: HashMap<String, CapabilityConfiguration>,
    mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HandshakeRequestBody {
    client_id: String,
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HandshakeResponse {
    private: String,
    certificate: String,
    id: String,
}

pub fn generate_authorization_header() -> String {
    let certs = {
        let db = borrow_db_checked();
        db.auth.clone().unwrap()
    };

    let nonce = Utc::now().timestamp_millis().to_string();

    let signature = sign_nonce(certs.private, nonce.clone()).unwrap();

    format!("Nonce {} {} {}", certs.client_id, nonce, signature)
}

pub async fn fetch_user() -> Result<User, RemoteAccessError> {
    let response = make_authenticated_get(generate_url(&["/api/v1/client/user"], &[])?).await?;
    if response.status() != 200 {
        let err: DropServerError = response.json().await?;
        warn!("{err:?}");

        if err.status_message == "Nonce expired" {
            return Err(RemoteAccessError::OutOfSync);
        }

        return Err(RemoteAccessError::InvalidResponse(err));
    }

    response
        .json::<User>()
        .await
        .map_err(std::convert::Into::into)
}

pub async fn recieve_handshake_logic(path: String) -> Result<(), RemoteAccessError> {
    let path_chunks: Vec<&str> = path.split('/').collect();
    if path_chunks.len() != 3 {
//        app.emit("auth/failed", ()).unwrap();
        return Err(RemoteAccessError::HandshakeFailed(
            "failed to parse token".to_string(),
        ));
    }

    let base_url = {
        let handle = borrow_db_checked();
        Url::parse(handle.base_url.as_str())?
    };

    let client_id = path_chunks.get(1).unwrap();
    let token = path_chunks.get(2).unwrap();
    let body = HandshakeRequestBody {
        client_id: (*client_id).to_string(),
        token: (*token).to_string(),
    };

    let endpoint = base_url.join("/api/v1/client/auth/handshake")?;
    let client = DROP_CLIENT_ASYNC.clone();
    let response = client.post(endpoint).json(&body).send().await?;
    debug!("handshake responsded with {}", response.status().as_u16());
    if !response.status().is_success() {
        return Err(RemoteAccessError::InvalidResponse(response.json().await?));
    }
    let response_struct: HandshakeResponse = response.json().await?;

    {
        let mut handle = borrow_db_mut_checked();
        handle.auth = Some(DatabaseAuth {
            private: response_struct.private,
            cert: response_struct.certificate,
            client_id: response_struct.id,
            web_token: None, // gets created later
        });
    }

    let web_token = {
        let header = generate_authorization_header();
        let token = client
            .post(base_url.join("/api/v1/client/user/webtoken").unwrap())
            .header("Authorization", header)
            .send()
            .await
            .unwrap();

        token.text().await.unwrap()
    };

    let mut handle = borrow_db_mut_checked();
    let mut_auth = handle.auth.as_mut().unwrap();
    mut_auth.web_token = Some(web_token);

    Ok(())
}

pub fn auth_initiate_logic(mode: String) -> Result<String, RemoteAccessError> {
    let base_url = {
        let db_lock = borrow_db_checked();
        Url::parse(&db_lock.base_url.clone())?
    };

    let hostname = gethostname();

    let endpoint = base_url.join("/api/v1/client/auth/initiate")?;
    let body = InitiateRequestBody {
        name: format!("{} (Desktop)", hostname.into_string().unwrap()),
        platform: env::consts::OS.to_string(),
        capabilities: HashMap::from([
            ("peerAPI".to_owned(), CapabilityConfiguration {}),
            ("cloudSaves".to_owned(), CapabilityConfiguration {}),
        ]),
        mode,
    };

    let client = DROP_CLIENT_SYNC.clone();
    let response = client.post(endpoint.to_string()).json(&body).send()?;

    if response.status() != 200 {
        let data: DropServerError = response.json()?;
        error!("could not start handshake: {}", data.status_message);

        return Err(RemoteAccessError::HandshakeFailed(data.status_message));
    }

    let response = response.text()?;

    Ok(response)
}