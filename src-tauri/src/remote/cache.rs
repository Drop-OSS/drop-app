use crate::{database::db::borrow_db_checked, error::remote_access_error::RemoteAccessError};
use cacache::Integrity;
use http::{header::CONTENT_TYPE, response::Builder as ResponseBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{UriSchemeContext, UriSchemeResponder};

use super::{auth::generate_authorization_header, requests::make_request};

#[macro_export]
macro_rules! offline {
    ($var:expr, $func1:expr, $func2:expr, $( $arg:expr ),* ) => {

        if crate::borrow_db_checked().settings.force_offline || $var.lock().unwrap().status == crate::AppStatus::Offline {
            $func2( $( $arg ), *)
        } else {
            $func1( $( $arg ), *)
        }
    }
}

pub fn cache_object<'a, K: AsRef<str>, D: Serialize + DeserializeOwned>(
    key: K,
    data: &D,
) -> Result<Integrity, RemoteAccessError> {
    let bytes = serde_json::to_vec(data).unwrap();
    cacache::write_sync(&borrow_db_checked().cache_dir, key, bytes)
        .map_err(|e| RemoteAccessError::Cache(e))
}
pub fn get_cached_object<'a, K: AsRef<str>, D: Serialize + DeserializeOwned>(
    key: K,
) -> Result<D, RemoteAccessError> {
    let bytes = cacache::read_sync(&borrow_db_checked().cache_dir, key)
        .map_err(|e| RemoteAccessError::Cache(e))?;
    let data = serde_json::from_slice::<D>(&bytes).unwrap();
    Ok(data)
}

pub fn fetch_object(
    request: http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    // Drop leading /
    let object_id = &request.uri().path()[1..];

    let header = generate_authorization_header();
    let client: reqwest::blocking::Client = reqwest::blocking::Client::new();
    let response = make_request(&client, &["/api/v1/client/object/", object_id], &[], |f| {
        f.header("Authorization", header)
    })
    .unwrap()
    .send();
    if response.is_err() {
        let data = get_cached_object::<&str, ObjectCache>(object_id);

        match data {
            Ok(data) => responder.respond(data.into()),
            Err(_) => todo!(),
        }
        return;
    }
    let response = response.unwrap();

    let resp_builder = ResponseBuilder::new().header(
        CONTENT_TYPE,
        response.headers().get("Content-Type").unwrap(),
    );
    let data = Vec::from(response.bytes().unwrap());
    let resp = resp_builder.body(data).unwrap();
    cache_object::<&str, ObjectCache>(object_id, &resp.clone().into()).unwrap();

    responder.respond(resp);
}
pub fn fetch_object_offline(
    request: http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let object_id = &request.uri().path()[1..];
    let data = get_cached_object::<&str, ObjectCache>(object_id);

    match data {
        Ok(data) => responder.respond(data.into()),
        Err(_) => todo!(),
    }
}
#[derive(Serialize, Deserialize)]
pub struct ObjectCache {
    content_type: String,
    body: Vec<u8>,
}

impl From<Response<Vec<u8>>> for ObjectCache {
    fn from(value: Response<Vec<u8>>) -> Self {
        ObjectCache {
            content_type: value
                .headers()
                .get(CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            body: value.body().clone(),
        }
    }
}
impl From<ObjectCache> for Response<Vec<u8>> {
    fn from(value: ObjectCache) -> Self {
        let resp_builder = ResponseBuilder::new().header(CONTENT_TYPE, value.content_type);
        resp_builder.body(value.body).unwrap()
    }
}
