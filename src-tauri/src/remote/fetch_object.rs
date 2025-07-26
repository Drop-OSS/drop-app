use http::{header::CONTENT_TYPE, response::Builder as ResponseBuilder};
use log::warn;
use tauri::UriSchemeResponder;

use super::{
    auth::generate_authorization_header,
    cache::{ObjectCache, cache_object, get_cached_object},
    requests::make_request,
};

pub async fn fetch_object(request: http::Request<Vec<u8>>, responder: UriSchemeResponder) {
    // Drop leading /
    let object_id = &request.uri().path()[1..];

    let cache_result = get_cached_object::<&str, ObjectCache>(object_id).await;
    if let Ok(cache_result) = &cache_result
        && !cache_result.has_expired()
    {
        responder.respond(cache_result.into());
        return;
    }

    let header = generate_authorization_header().await;
    let client = reqwest::Client::new();
    let response = make_request(
        &client,
        &["/api/v1/client/object/", object_id],
        &[],
        async |f| f.header("Authorization", header),
    )
    .await
    .unwrap()
    .send()
    .await;
    if response.is_err() {
        match cache_result {
            Ok(cache_result) => responder.respond(cache_result.into()),
            Err(e) => {
                warn!("{e}")
            }
        }
        return;
    }
    let response = response.unwrap();

    let resp_builder = ResponseBuilder::new().header(
        CONTENT_TYPE,
        response.headers().get("Content-Type").unwrap(),
    );
    let data = Vec::from(response.bytes().await.unwrap());
    let resp = resp_builder.body(data).unwrap();
    if cache_result.is_err() || cache_result.unwrap().has_expired() {
        cache_object::<&str, ObjectCache>(object_id, &resp.clone().into())
            .await
            .unwrap();
    }

    responder.respond(resp);
}
pub async fn fetch_object_offline(request: http::Request<Vec<u8>>, responder: UriSchemeResponder) {
    let object_id = &request.uri().path()[1..];
    let data = get_cached_object::<&str, ObjectCache>(object_id).await;

    match data {
        Ok(data) => responder.respond(data.into()),
        Err(e) => warn!("{e}"),
    }
}
