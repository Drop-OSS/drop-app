use std::{collections::HashMap, env, sync::Mutex};

use chrono::Utc;
use droplet_rs::ssl::sign_nonce;
use gethostname::gethostname;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{
    database::{
        db::{borrow_db_checked, borrow_db_mut_checked, save_db, DatabaseImpls},
        models::data::DatabaseAuth,
    },
    error::{drop_server_error::DropServerError, remote_access_error::RemoteAccessError},
    AppState, AppStatus, User, DB,
};

use super::{
    cache::{cache_object, get_cached_object},
    requests::make_request,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityConfiguration {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitiateRequestBody {
    name: String,
    platform: String,
    capabilities: HashMap<String, CapabilityConfiguration>,
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

pub fn fetch_user() -> Result<User, RemoteAccessError> {
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
    if !response.status().is_success() {
        return Err(RemoteAccessError::InvalidResponse(response.json()?));
    }
    let response_struct: HandshakeResponse = response.json()?;

    {
        let mut handle = borrow_db_mut_checked();
        handle.auth = Some(DatabaseAuth {
            private: response_struct.private,
            cert: response_struct.certificate,
            client_id: response_struct.id,
            web_token: None, // gets created later
        });
        drop(handle);
        save_db();
    }

    let web_token = {
        let header = generate_authorization_header();
        let token = client
            .post(base_url.join("/api/v1/client/user/webtoken").unwrap())
            .header("Authorization", header)
            .send()
            .unwrap();

        token.text().unwrap()
    };

    let mut handle = borrow_db_mut_checked();
    let mut_auth = handle.auth.as_mut().unwrap();
    mut_auth.web_token = Some(web_token);
    drop(handle);
    save_db();

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

    let hostname = gethostname();

    let endpoint = base_url.join("/api/v1/client/auth/initiate")?;
    let body = InitiateRequestBody {
        name: format!("{} (Desktop)", hostname.into_string().unwrap()),
        platform: env::consts::OS.to_string(),
        capabilities: HashMap::from([
            ("peerAPI".to_owned(), CapabilityConfiguration {}),
            ("cloudSaves".to_owned(), CapabilityConfiguration {}),
        ]),
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
            Err(RemoteAccessError::FetchError(_)) => {
                let user = get_cached_object::<String, User>("user".to_owned()).unwrap();
                return (AppStatus::Offline, Some(user));
            }
            Err(_) => return (AppStatus::SignedInNeedsReauth, None),
        };
        cache_object("user", &user_result).unwrap();
        return (AppStatus::SignedIn, Some(user_result));
    }

    (AppStatus::SignedOut, None)
}
