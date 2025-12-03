
use database::{DB, interface::DatabaseImpls};
use log::info;
use url::Url;

use crate::{
    auth::generate_authorization_header, error::RemoteAccessError, utils::DROP_CLIENT_ASYNC,
};

pub fn generate_url<T: AsRef<str>>(
    path_components: &[T],
    query: &[(T, T)],
) -> Result<Url, RemoteAccessError> {
    let components = path_components.iter().map(|v| v.as_ref()).map(|v| v.trim_matches('/')).collect::<Vec<&str>>();
    let mut base_url = DB
        .fetch_base_url()
        .join(&components.join("/"))?;
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
