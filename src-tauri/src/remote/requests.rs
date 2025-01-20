use reqwest::blocking::{Client, RequestBuilder};

use crate::{database::db::DatabaseImpls, error::remote_access_error::RemoteAccessError, DB};

pub fn make_request<T: AsRef<str>, F: FnOnce(RequestBuilder) -> RequestBuilder>(
    client: &Client,
    path_components: &[T],
    query: &[(T, T)],
    f: F,
) -> Result<RequestBuilder, RemoteAccessError> {
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
    let response = client.get(base_url);
    Ok(f(response))
}
