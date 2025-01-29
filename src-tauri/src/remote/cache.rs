use std::collections::HashMap;

use http::Version;
use reqwest::blocking::{Request, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
struct StoredResponse {
    body: Vec<u8>,
    headers: HashMap<String, String>,
    status: u16,
    url: Url,
}
// HTTP version enum in the http crate does not support serde, hence the modified copy.
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
enum HttpVersion {
    #[serde(rename = "HTTP/0.9")]
    Http09,
    #[serde(rename = "HTTP/1.0")]
    Http10,
    #[serde(rename = "HTTP/1.1")]
    Http11,
    #[serde(rename = "HTTP/2.0")]
    H2,
    #[serde(rename = "HTTP/3.0")]
    H3,
}
impl From<HttpVersion> for Version {
    fn from(value: HttpVersion) -> Self {
        match value {
            HttpVersion::Http09 => Version::HTTP_09,
            HttpVersion::Http10 => Version::HTTP_10,
            HttpVersion::Http11 => Version::HTTP_11,
            HttpVersion::H2 => Version::HTTP_2,
            HttpVersion::H3 => Version::HTTP_3,
        }
    }
}
impl From<Version> for HttpVersion {
    fn from(value: Version) -> Self {
        match value {
            Version::HTTP_09 => HttpVersion::Http09,
            Version::HTTP_10 => HttpVersion::Http10,
            Version::HTTP_11 => HttpVersion::Http11,
            Version::HTTP_2 => HttpVersion::H2,
            Version::HTTP_3 => HttpVersion::H3,
            _ => unreachable!()
        }
    }
}

pub trait Cache {
    fn send_cache(req: RequestBuilder) -> Result<Response, reqwest::Error>;
}

impl Cache for Request {
    fn send_cache(req: RequestBuilder) -> Result<Response, reqwest::Error> {
        let res = req.send()?;
        let mut headers = HashMap::new();
        for header in res.headers() {
            headers.insert(header.0.as_str().to_owned(), header.1.to_str().unwrap().to_owned());
        }
        let status = res.status().as_u16();
        let url = res.url().clone();
        let version: HttpVersion = res.version().into();
        let body: Vec<u8> = res.bytes()?.to_vec();
        
    }
}