use std::ops::Deref;

use reqwest::blocking::{Client, RequestBuilder, Response};
use url::{ParseError, Url};

use crate::{database::db::DatabaseImpls, error::remote_access_error::RemoteAccessError, DB};

pub fn make_request<'a, T: AsRef<str>, F: FnOnce(RequestBuilder) -> RequestBuilder>(
    client: &Client,
    endpoints: &[T],
    params: &[(T, T)],
    f: F,
) -> Result<RequestBuilder, RemoteAccessError> {
    let mut base_url = DB.fetch_base_url();
    for endpoint in endpoints {
        base_url = base_url.join(endpoint.as_ref())?;
    }
    {
        let mut queries = base_url.query_pairs_mut();
        for (param, val) in params {
            queries.append_pair(param.as_ref(), val.as_ref());
        }
    }
    let response = client.get(base_url);
    Ok(f(response))
}
