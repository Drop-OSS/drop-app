use core::time;
use std::{
    borrow::{Borrow, BorrowMut},
    env,
    fmt::format,
    sync::Mutex, time::{SystemTime, UNIX_EPOCH},
};

use log::{info, warn};
use openssl::{
    ec::EcKey,
    hash::MessageDigest,
    pkey::PKey,
    sign::{self, Signer},
};
use serde::{Deserialize, Serialize};
use tauri::{http::response, App, AppHandle, Emitter, EventLoopMessage, Manager, Wry};
use url::Url;
use uuid::Uuid;

use crate::{db::{fetch_base_url, DatabaseAuth}, AppState, AppStatus, User, DB};

#[derive(Serialize)]
struct InitiateRequestBody {
    name: String,
    platform: String,
}

#[derive(Serialize)]
struct HandshakeRequestBody {
    clientId: String,
    token: String,
}

#[derive(Deserialize)]
struct HandshakeResponse {
    private: String,
    certificate: String,
    id: String,
}

macro_rules! unwrap_or_return {
    ( $e:expr, $app:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                $app.emit("auth/failed", ()).unwrap();
                return;
            }
        }
    };
}

pub fn sign_nonce(private_key: String, nonce: String) -> Result<String, ()> {
    let client_private_key = EcKey::private_key_from_pem(private_key.as_bytes()).unwrap();
    let pkey_private_key = PKey::from_ec_key(client_private_key).unwrap();

    let mut signer = Signer::new(MessageDigest::sha256(), &pkey_private_key).unwrap();
    signer.update(nonce.as_bytes()).unwrap();
    let signature = signer.sign_to_vec().unwrap();

    let hex_signature = hex::encode(signature);

    return Ok(hex_signature);
}

pub fn generate_authorization_header() -> String {
    let certs = {
        let db = DB.borrow_data().unwrap();
        db.auth.clone().unwrap()
    };

    let start = SystemTime::now();
    let timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let nonce = timestamp.as_millis().to_string();

    let signature = sign_nonce(certs.private, nonce.clone()).unwrap();

    return format!("Nonce {} {} {}", certs.clientId, nonce, signature);
}

pub fn fetch_user() -> Result<User, ()> {
    let base_url = fetch_base_url();

    let endpoint = base_url.join("/api/v1/client/user").unwrap();
    let header = generate_authorization_header();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(endpoint.to_string())
        .header("Authorization", header)
        .send()
        .unwrap();

    if response.status() != 200 {
        warn!("Failed to fetch user: {}", response.status());
        return Err(());
    }

    let user = response.json::<User>().unwrap();

    return Ok(user);
}

pub fn recieve_handshake(app: AppHandle, path: String) {
    // Tell the app we're processing
    app.emit("auth/processing", ()).unwrap();

    let path_chunks: Vec<&str> = path.split("/").collect();
    if path_chunks.len() != 3 {
        app.emit("auth/failed", ()).unwrap();
        return;
    }

    let base_url = {
        let handle = DB.borrow_data().unwrap();
        Url::parse(handle.base_url.as_str()).unwrap()
    };

    let client_id = path_chunks.get(1).unwrap();
    let token = path_chunks.get(2).unwrap();
    let body = HandshakeRequestBody {
        clientId: client_id.to_string(),
        token: token.to_string(),
    };

    let endpoint = unwrap_or_return!(base_url.join("/api/v1/client/auth/handshake"), app);
    let client = reqwest::blocking::Client::new();
    let response = unwrap_or_return!(client.post(endpoint).json(&body).send(), app);
    info!("server responded with {}", response.status());
    let response_struct = unwrap_or_return!(response.json::<HandshakeResponse>(), app);

    {
        let mut handle = DB.borrow_data_mut().unwrap();
        handle.auth = Some(DatabaseAuth {
            private: response_struct.private,
            cert: response_struct.certificate,
            clientId: response_struct.id,
        });
        drop(handle);
        DB.save().unwrap();
    }

    {
        let app_state = app.state::<Mutex<AppState>>();
        let mut app_state_handle = app_state.lock().unwrap();
        app_state_handle.status = AppStatus::SignedIn;
        app_state_handle.user = Some(fetch_user().unwrap());
    }

    app.emit("auth/finished", ()).unwrap();
}

#[tauri::command]
pub async fn auth_initiate<'a>() -> Result<(), String> {
    let base_url = {
        let db_lock = DB.borrow_data().unwrap();
        Url::parse(&db_lock.base_url.clone()).unwrap()
    };

    let endpoint = base_url.join("/api/v1/client/auth/initiate").unwrap();
    let body = InitiateRequestBody {
        name: format!("Drop Desktop Client"),
        platform: env::consts::OS.to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(endpoint.to_string())
        .json(&body)
        .send()
        .await
        .unwrap();

    if response.status() != 200 {
        return Err("Failed to create redirect URL. Please try again later.".to_string());
    }

    let redir_url = response.text().await.unwrap();
    let complete_redir_url = base_url.join(&redir_url).unwrap();

    info!("opening web browser to continue authentication");
    webbrowser::open(&complete_redir_url.to_string()).unwrap();

    return Ok(());
}

pub fn setup() -> Result<(AppStatus, Option<User>), ()> {
    let data = DB.borrow_data().unwrap();

    // If we have certs, exit for now
    if data.auth.is_some() {
        let user_result = fetch_user();
        if user_result.is_err() {
            return Ok((AppStatus::SignedInNeedsReauth, None));
        }
        return Ok((AppStatus::SignedIn, Some(user_result.unwrap())));
    }

    drop(data);

    return Ok((AppStatus::SignedOut, None));
}
