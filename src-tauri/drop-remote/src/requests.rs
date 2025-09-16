use drop_errors::remote_access_error::RemoteAccessError;
use url::Url;

use crate::{auth::generate_authorization_header, utils::DROP_CLIENT_ASYNC, DropRemoteContext};

pub fn generate_url<T: AsRef<str>>(
    context: &DropRemoteContext,
    path_components: &[T],
    query: &[(T, T)],
) -> Result<Url, RemoteAccessError> {
    let mut base_url = context.base_url.clone();
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

pub async fn make_authenticated_get(context: &DropRemoteContext, url: Url) -> Result<reqwest::Response, reqwest::Error> {
    DROP_CLIENT_ASYNC
        .get(url)
        .header("Authorization", generate_authorization_header(context))
        .send()
        .await
}
