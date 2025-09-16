use std::{collections::HashMap, env, sync::Mutex};

use chrono::Utc;
use drop_errors::{drop_server_error::ServerError, remote_access_error::RemoteAccessError};
use droplet_rs::ssl::sign_nonce;
use gethostname::gethostname;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    requests::make_authenticated_get, utils::{DROP_CLIENT_ASYNC, DROP_CLIENT_SYNC}, DropRemoteAuth, DropRemoteContext
};

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

pub fn generate_authorization_header(context: &DropRemoteContext) -> String {
    let auth = if let Some(auth) = &context.auth {
        auth
    } else {
        return "".to_owned();
    };
    let nonce = Utc::now().timestamp_millis().to_string();

    let signature = sign_nonce(auth.private.clone(), nonce.clone()).unwrap();

    format!("Nonce {} {} {}", auth.client_id, nonce, signature)
}

pub async fn fetch_user(context: &DropRemoteContext) -> Result<Vec<u8>, RemoteAccessError> {
    let response =
        make_authenticated_get(context, generate_url(context, &["/api/v1/client/user"], &[])?).await?;
    if response.status() != 200 {
        let err: ServerError = response.json().await?;
        warn!("{err:?}");

        if err.status_message == "Nonce expired" {
            return Err(RemoteAccessError::OutOfSync);
        }

        return Err(RemoteAccessError::InvalidResponse(err));
    }

    response
        .bytes()
        .await
        .map_err(std::convert::Into::into)
        .map(|v| v.to_vec())
}

pub async fn recieve_handshake_logic(
    context: &mut DropRemoteContext,
    path: String,
) -> Result<(), RemoteAccessError> {
    let path_chunks: Vec<&str> = path.split('/').collect();
    if path_chunks.len() != 3 {
        //        app.emit("auth/failed", ()).unwrap();
        return Err(RemoteAccessError::HandshakeFailed(
            "failed to parse token".to_string(),
        ));
    }

    let client_id = path_chunks.get(1).unwrap();
    let token = path_chunks.get(2).unwrap();
    let body = HandshakeRequestBody {
        client_id: (*client_id).to_string(),
        token: (*token).to_string(),
    };

    let endpoint = generate_url(context, &["/api/v1/client/auth/handshake"], &[])?;
    let client = DROP_CLIENT_ASYNC.clone();
    let response = client.post(endpoint).json(&body).send().await?;
    debug!("handshake responsded with {}", response.status().as_u16());
    if !response.status().is_success() {
        return Err(RemoteAccessError::InvalidResponse(response.json().await?));
    }
    let response_struct: HandshakeResponse = response.json().await?;

    let web_token = {
        let header = generate_authorization_header(context);
        let token = client
            .post(generate_url(context, &["/api/v1/client/user/webtoken"], &[])?)
            .header("Authorization", header)
            .send()
            .await
            .unwrap();

        token.text().await.unwrap()
    };

    context.auth = Some(DropRemoteAuth {
        private: response_struct.private,
        cert: response_struct.certificate,
        client_id: response_struct.id,
        web_token: web_token,
    });

    Ok(())
}

pub fn auth_initiate_logic(context: &DropRemoteContext, mode: String) -> Result<String, RemoteAccessError> {
    let hostname = gethostname();

    let endpoint = generate_url(context, &["/api/v1/client/auth/initiate"], &[])?;
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
        let data: ServerError = response.json()?;
        error!("could not start handshake: {}", data.status_message);

        return Err(RemoteAccessError::HandshakeFailed(data.status_message));
    }

    let response = response.text()?;

    Ok(response)
}
