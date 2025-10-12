use std::str::FromStr;

use database::borrow_db_checked;
use http::{Request, Response, StatusCode, Uri, uri::PathAndQuery};
use log::{error, warn};
use tauri::UriSchemeResponder;
use utils::webbrowser_open::webbrowser_open;

use crate::utils::DROP_CLIENT_SYNC;

pub async fn handle_server_proto_offline_wrapper(
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    responder.respond(match handle_server_proto_offline(request).await {
        Ok(res) => res,
        Err(_) => unreachable!(),
    });
}

pub async fn handle_server_proto_offline(
    _request: Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, StatusCode> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Vec::new())
        .expect("Failed to build error response for proto offline"))
}

pub async fn handle_server_proto_wrapper(request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    match handle_server_proto(request).await {
        Ok(r) => responder.respond(r),
        Err(e) => {
            warn!("Cache error: {e}");
            responder.respond(
                Response::builder()
                    .status(e)
                    .body(Vec::new())
                    .expect("Failed to build error response"),
            );
        }
    }
}

async fn handle_server_proto(request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, StatusCode> {
    let db_handle = borrow_db_checked();
    let auth = match db_handle.auth.as_ref() {
        Some(auth) => auth,
        None => {
            error!("Could not find auth in database");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    let web_token = match &auth.web_token {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    let remote_uri = db_handle
        .base_url
        .parse::<Uri>()
        .expect("Failed to parse base url");

    let path = request.uri().path();

    let mut new_uri = request.uri().clone().into_parts();
    new_uri.path_and_query = Some(
        PathAndQuery::from_str(&format!("{path}?noWrapper=true"))
            .expect("Failed to parse request path in proto"),
    );
    new_uri.authority = remote_uri.authority().cloned();
    new_uri.scheme = remote_uri.scheme().cloned();
    let err_msg = &format!("Failed to build new uri from parts {new_uri:?}");
    let new_uri = Uri::from_parts(new_uri).expect(err_msg);

    let whitelist_prefix = ["/store", "/api", "/_", "/fonts"];

    if whitelist_prefix.iter().all(|f| !path.starts_with(f)) {
        webbrowser_open(new_uri.to_string());
        return Ok(Response::new(Vec::new()));
    }

    let client = DROP_CLIENT_SYNC.clone();
    let response = match client
        .request(request.method().clone(), new_uri.to_string())
        .header("Authorization", format!("Bearer {web_token}"))
        .headers(request.headers().clone())
        .send()
    {
        Ok(response) => response,
        Err(e) => {
            warn!("Could not send response. Got {e} when sending");
            return Err(e.status().unwrap_or(StatusCode::BAD_REQUEST));
        }
    };

    let response_status = response.status();
    let response_body = match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => return Err(e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let http_response = Response::builder()
        .status(response_status)
        .body(response_body.to_vec())
        .expect("Failed to build server proto response");

    Ok(http_response)
}
