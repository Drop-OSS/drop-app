use std::{env, sync::Mutex};

use chrono::Utc;
use log::{debug, error, warn};
use openssl::{ec::EcKey, hash::MessageDigest, pkey::PKey, sign::Signer};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{
    database::db::{
        borrow_db_checked, borrow_db_mut_checked, save_db, DatabaseAuth, DatabaseImpls,
    },
    error::{drop_server_error::DropServerError, remote_access_error::RemoteAccessError},
    AppState, AppStatus, User, DB,
};

use super::requests::make_request;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitiateRequestBody {
    name: String,
    platform: String,
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

// TODO: Change return value on Err
pub fn sign_nonce(private_key: String, nonce: String) -> Result<String, ()> {
    let client_private_key = EcKey::private_key_from_pem(private_key.as_bytes()).unwrap();
    let pkey_private_key = PKey::from_ec_key(client_private_key).unwrap();

    let mut signer = Signer::new(MessageDigest::sha256(), &pkey_private_key).unwrap();
    signer.update(nonce.as_bytes()).unwrap();
    let signature = signer.sign_to_vec().unwrap();

    let hex_signature = hex::encode(signature);

    Ok(hex_signature)
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

pub fn fetch_user() -> Result<User, RemoteAccessError> {
    let base_url = DB.fetch_base_url();

    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = make_request(&client, &["/api/v1/client/user"], &[], |f| {
        f.header("Authorization", header)
    })?
    .send()?;
    if response.status() != 200 {
        let err: DropServerError = response.json()?;
        warn!("{:?}", err);

        if err.status_message == "Nonce expired" {
            return Err(RemoteAccessError::OutOfSync);
        }

        return Err(RemoteAccessError::InvalidResponse(err));
    }

    response.json::<User>().map_err(|e| e.into())
}

fn recieve_handshake_logic(app: &AppHandle, path: String) -> Result<(), RemoteAccessError> {
    let path_chunks: Vec<&str> = path.split("/").collect();
    if path_chunks.len() != 3 {
        app.emit("auth/failed", ()).unwrap();
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
        client_id: client_id.to_string(),
        token: token.to_string(),
    };

    let endpoint = base_url.join("/api/v1/client/auth/handshake")?;
    let client = reqwest::blocking::Client::new();
    let response = client.post(endpoint).json(&body).send()?;
    debug!("handshake responsded with {}", response.status().as_u16());
    let response_struct: HandshakeResponse = response.json()?;

    {
        let mut handle = borrow_db_mut_checked();
        handle.auth = Some(DatabaseAuth {
            private: response_struct.private,
            cert: response_struct.certificate,
            client_id: response_struct.id,
        });
        drop(handle);
        save_db();
    }

    {
        let app_state = app.state::<Mutex<AppState>>();
        let mut app_state_handle = app_state.lock().unwrap();
        app_state_handle.status = AppStatus::SignedIn;
        app_state_handle.user = Some(fetch_user()?);
    }

    Ok(())
}

pub fn recieve_handshake(app: AppHandle, path: String) {
    // Tell the app we're processing
    app.emit("auth/processing", ()).unwrap();

    let handshake_result = recieve_handshake_logic(&app, path);
    if let Err(e) = handshake_result {
        warn!("error with authentication: {}", e);
        app.emit("auth/failed", e.to_string()).unwrap();
        return;
    }

    app.emit("auth/finished", ()).unwrap();
}

pub fn auth_initiate_logic() -> Result<(), RemoteAccessError> {
    let base_url = {
        let db_lock = borrow_db_checked();
        Url::parse(&db_lock.base_url.clone())?
    };

    let endpoint = base_url.join("/api/v1/client/auth/initiate")?;
    let body = InitiateRequestBody {
        name: "Drop Desktop Client".to_string(),
        platform: env::consts::OS.to_string(),
    };

    let client = reqwest::blocking::Client::new();
    let response = client.post(endpoint.to_string()).json(&body).send()?;

    if response.status() != 200 {
        let data: DropServerError = response.json()?;
        error!("could not start handshake: {}", data.status_message);

        return Err(RemoteAccessError::HandshakeFailed(data.status_message));
    }

    let redir_url = response.text()?;
    let complete_redir_url = base_url.join(&redir_url)?;

    debug!("opening web browser to continue authentication");
    webbrowser::open(complete_redir_url.as_ref()).unwrap();

    Ok(())
}

pub fn setup() -> (AppStatus, Option<User>) {
    let data = borrow_db_checked();
    let auth = data.auth.clone();
    drop(data);

    if auth.is_some() {
        let user_result = match fetch_user() {
            Ok(data) => data,
            Err(RemoteAccessError::FetchError(_)) => return (AppStatus::Offline, None),
            Err(_) => return (AppStatus::SignedInNeedsReauth, None),
        };
        return (AppStatus::SignedIn, Some(user_result));
    }

    (AppStatus::SignedOut, None)
}
