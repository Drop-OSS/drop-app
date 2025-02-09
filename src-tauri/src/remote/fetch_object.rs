use http::{header::CONTENT_TYPE, response::Builder as ResponseBuilder};
use log::warn;
use tauri::UriSchemeResponder;

use super::{auth::generate_authorization_header, cache::{cache_object, get_cached_object, ObjectCache}, requests::make_request};

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
            Err(e) => {
                warn!("{}", e)
            },
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
        Err(e) => warn!("{}", e),
    }
}
