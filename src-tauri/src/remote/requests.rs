use reqwest::blocking::{Client, RequestBuilder};
use url::Url;

use crate::{
    DB,
    database::db::DatabaseImpls,
    error::remote_access_error::RemoteAccessError,
    remote::{auth::generate_authorization_header, utils::DROP_CLIENT_ASYNC},
};

pub fn generate_url<T: AsRef<str>>(
    path_components: &[T],
    query: &[(T, T)],
) -> Result<Url, RemoteAccessError> {
    let mut base_url = DB.fetch_base_url();
    for endpoint in path_components {
        base_url = base_url.join(endpoint.as_ref())?;
    }
    {
        let mut queries = base_url.query_pairs_mut();
        for (param, val) in query {
            queries.append_pair(param.as_ref(), val.as_ref());
        }
    }
    Ok(base_url)
}

pub async fn make_authenticated_get(url: Url) -> Result<reqwest::Response, reqwest::Error> {
    DROP_CLIENT_ASYNC
        .get(url)
        .header("Authorization", generate_authorization_header())
        .send()
        .await
}
