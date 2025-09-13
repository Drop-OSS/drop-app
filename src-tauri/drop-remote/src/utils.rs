use std::{
    fs::{self, File},
    io::Read,
    sync::LazyLock,
};

use drop_database::db::DATA_ROOT_DIR;
use log::{debug, info};
use reqwest::Certificate;
use serde::Deserialize;

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