use std::{
    fs::{self, File},
    io::Read,
    sync::Mutex,
};

use log::{debug, warn};
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

pub fn get_client() -> reqwest::blocking::ClientBuilder {
    let mut client = reqwest::blocking::ClientBuilder::new();
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
                    }
                    Err(_) => todo!(),
                }
            }
        }
        Err(e) => {
            debug!("Not loading certificates due to error {e}");
        }
    };
    for cert in certs {
        client = client.add_root_certificate(cert);
    }
    return client;
}
pub fn use_remote_logic(
    url: String,
    state: tauri::State<'_, Mutex<AppState<'_>>>,
) -> Result<(), RemoteAccessError> {
    debug!("connecting to url {url}");
    let base_url = Url::parse(&url)?;

    // Test Drop url
    let test_endpoint = base_url.join("/api/v1")?;
    let response = reqwest::blocking::get(test_endpoint.to_string())?;

    let result: DropHealthcheck = response.json()?;

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
