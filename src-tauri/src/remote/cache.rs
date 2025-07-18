use crate::{
    database::{db::borrow_db_checked, models::data::Database},
    error::remote_access_error::RemoteAccessError,
};
use cacache::Integrity;
use http::{header::CONTENT_TYPE, response::Builder as ResponseBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

#[macro_export]
macro_rules! offline {
    ($var:expr, $func1:expr, $func2:expr, $( $arg:expr ),* ) => {

        if $crate::borrow_db_checked().settings.force_offline || $var.lock().unwrap().status == $crate::AppStatus::Offline {
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
    let bytes = serde_binary::to_vec(data, Endian::Little).unwrap();
    cacache::write_sync(&borrow_db_checked().cache_dir, key, bytes)
        .map_err(RemoteAccessError::Cache)
}
pub fn get_cached_object<'a, K: AsRef<str>, D: Serialize + DeserializeOwned>(
    key: K,
) -> Result<D, RemoteAccessError> {
    get_cached_object_db::<K, D>(key, &borrow_db_checked())
}
pub fn get_cached_object_db<'a, K: AsRef<str>, D: Serialize + DeserializeOwned>(
    key: K,
    db: &Database,
) -> Result<D, RemoteAccessError> {
    let bytes = cacache::read_sync(&db.cache_dir, key).map_err(RemoteAccessError::Cache)?;
    let data = serde_binary::from_slice::<D>(&bytes, Endian::Little).unwrap();
    Ok(data)
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
