use std::{
    fs::{self, File},
    io::Read,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use log::{debug, info, warn};
use reqwest::Certificate;
use serde::Deserialize;
use url::Url;

use crate::{
    AppState, AppStatus,
    database::db::{DATA_ROOT_DIR, borrow_db_mut_checked},
    error::remote_access_error::RemoteAccessError,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DropHealthcheck {
    app_name: String,
}

static DROP_CERT_BUNDLE: LazyLock<Vec<Certificate>> = LazyLock::new(fetch_certificates);
pub static DROP_CLIENT_SYNC: LazyLock<reqwest::blocking::Client> = LazyLock::new(get_client_sync);
pub static DROP_CLIENT_ASYNC: LazyLock<reqwest::Client> = LazyLock::new(get_client_async);
pub static DROP_CLIENT_WS_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(get_client_ws);

fn fetch_certificates() -> Vec<Certificate> {
    let certificate_dir = DATA_ROOT_DIR.join("certificates");

    let mut certs = Vec::new();
    match fs::read_dir(certificate_dir) {
        Ok(c) => {
            for entry in c {
                match entry {
                    Ok(c) => {
                        let mut buf = Vec::new();
                        File::open(c.path()).unwrap().read_to_end(&mut buf).unwrap();

                        for cert in Certificate::from_pem_bundle(&buf).unwrap() {
                            certs.push(cert);
                        }
                        info!(
                            "added {} certificate(s) from {}",
                            certs.len(),
                            c.file_name().into_string().unwrap()
                        );
                    }
                    Err(_) => todo!(),
                }
            }
        }
        Err(e) => {
            debug!("not loading certificates due to error: {e}");
        }
    };
    certs
}

pub fn get_client_sync() -> reqwest::blocking::Client {
    let mut client = reqwest::blocking::ClientBuilder::new();

    for cert in DROP_CERT_BUNDLE.iter() {
        client = client.add_root_certificate(cert.clone());
    }
    client.use_rustls_tls().build().unwrap()
}
pub fn get_client_async() -> reqwest::Client {
    let mut client = reqwest::ClientBuilder::new();

    for cert in DROP_CERT_BUNDLE.iter() {
        client = client.add_root_certificate(cert.clone());
    }
    client.use_rustls_tls().build().unwrap()
}
pub fn get_client_ws() -> reqwest::Client {
    let mut client = reqwest::ClientBuilder::new();

    for cert in DROP_CERT_BUNDLE.iter() {
        client = client.add_root_certificate(cert.clone());
    }
    client.use_rustls_tls().http1_only().build().unwrap()
}

pub async fn use_remote_logic(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), RemoteAccessError> {
    debug!("connecting to url {url}");
    let base_url = Url::parse(&url)?;

    // Test Drop url
    let test_endpoint = base_url.join("/api/v1")?;
    let client = DROP_CLIENT_ASYNC.clone();
    let response = client
        .get(test_endpoint.to_string())
        .timeout(Duration::from_secs(3))
        .send()
        .await?;

    let result: DropHealthcheck = response.json().await?;

    if result.app_name != "Drop" {
        warn!("user entered drop endpoint that connected, but wasn't identified as Drop");
        return Err(RemoteAccessError::InvalidEndpoint);
    }

    let mut app_state = state.lock().unwrap();
    app_state.status = AppStatus::SignedOut;
    drop(app_state);

    let mut db_state = borrow_db_mut_checked();
    db_state.base_url = base_url.to_string();

    Ok(())
}
