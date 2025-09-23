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
    database::db::{borrow_db_mut_checked, DATA_ROOT_DIR}, error::remote_access_error::RemoteAccessError, lock, AppState, AppStatus
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
                        match File::open(c.path()) {
                            Ok(f) => f,
                            Err(e) => {
                                warn!(
                                    "Failed to open file at {} with error {}",
                                    c.path().display(),
                                    e
                                );
                                continue;
                            }
                        }
                        .read_to_end(&mut buf)
                        .unwrap_or_else(|e| panic!(
                            "Failed to read to end of certificate file {} with error {}",
                            c.path().display(),
                            e
                        ));

                        match Certificate::from_pem_bundle(&buf) {
                            Ok(certificates) => {
                                for cert in certificates {
                                    certs.push(cert);
                                }
                                info!(
                                    "added {} certificate(s) from {}",
                                    certs.len(),
                                    c.file_name().display()
                                );
                            }
                            Err(e) => warn!(
                                "Invalid certificate file {} with error {}",
                                c.path().display(),
                                e
                            ),
                        }
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
    client.use_rustls_tls().build().expect("Failed to build synchronous client")
}
pub fn get_client_async() -> reqwest::Client {
    let mut client = reqwest::ClientBuilder::new();

    for cert in DROP_CERT_BUNDLE.iter() {
        client = client.add_root_certificate(cert.clone());
    }
    client.use_rustls_tls().build().expect("Failed to build asynchronous client")
}
pub fn get_client_ws() -> reqwest::Client {
    let mut client = reqwest::ClientBuilder::new();

    for cert in DROP_CERT_BUNDLE.iter() {
        client = client.add_root_certificate(cert.clone());
    }
    client
        .use_rustls_tls()
        .http1_only()
        .build()
        .expect("Failed to build websocket client")
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

    let mut app_state = lock!(state);
    app_state.status = AppStatus::SignedOut;
    drop(app_state);

    let mut db_state = borrow_db_mut_checked();
    db_state.base_url = base_url.to_string();

    Ok(())
}
