use std::str::FromStr;

use http::{uri::PathAndQuery, Request, Response, StatusCode, Uri};
use reqwest::blocking::Client;
use tauri::UriSchemeResponder;

use crate::database::db::borrow_db_checked;

pub fn handle_server_proto_offline(_request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let four_oh_four = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Vec::new())
        .unwrap();
    responder.respond(four_oh_four);
}

pub fn handle_server_proto(request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let db_handle = borrow_db_checked();
    let web_token = match &db_handle.auth.as_ref().unwrap().web_token {
        Some(e) => e,
        None => return,
    };
    let remote_uri = db_handle.base_url.parse::<Uri>().unwrap();

    let path = request.uri().path();

    let mut new_uri = request.uri().clone().into_parts();
    new_uri.path_and_query =
        Some(PathAndQuery::from_str(&format!("{path}?noWrapper=true")).unwrap());
    new_uri.authority = remote_uri.authority().cloned();
    new_uri.scheme = remote_uri.scheme().cloned();
    let new_uri = Uri::from_parts(new_uri).unwrap();

    let whitelist_prefix = ["/store", "/api", "/_", "/fonts"];

    if whitelist_prefix
        .iter()
        .all(|f| !path.starts_with(f))
    {
        webbrowser::open(&new_uri.to_string()).unwrap();
        return;
    }

    let client = Client::new();
    let response = client
        .request(request.method().clone(), new_uri.to_string())
        .header("Authorization", format!("Bearer {web_token}"))
        .headers(request.headers().clone())
        .send()
        .unwrap();

    let response_status = response.status();
    let response_body = response.bytes().unwrap();

    let http_response = Response::builder()
        .status(response_status)
        .body(response_body.to_vec())
        .unwrap();

    responder.respond(http_response);
}
