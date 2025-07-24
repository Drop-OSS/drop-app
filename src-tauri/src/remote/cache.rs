use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

use crate::{
    database::{db::borrow_db_checked, models::data::Database},
    error::remote_access_error::RemoteAccessError,
};
use bitcode::{Decode, DecodeOwned, Encode};
use cacache::Integrity;
use http::{Response, header::CONTENT_TYPE, response::Builder as ResponseBuilder};
use log::info;

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

pub fn cache_object<K: AsRef<str>, D: Encode>(
    key: K,
    data: &D,
) -> Result<Integrity, RemoteAccessError> {
    let bytes = bitcode::encode(data);
    cacache::write_sync(&borrow_db_checked().cache_dir, key, bytes)
        .map_err(RemoteAccessError::Cache)
}
pub fn get_cached_object<K: AsRef<str> + Display, D: Encode + DecodeOwned>(
    key: K,
) -> Result<D, RemoteAccessError> {
    get_cached_object_db::<K, D>(key, &borrow_db_checked())
}
pub fn get_cached_object_db<K: AsRef<str> + Display, D: DecodeOwned>(
    key: K,
    db: &Database,
) -> Result<D, RemoteAccessError> {
    let bytes = cacache::read_sync(&db.cache_dir, &key).map_err(RemoteAccessError::Cache)?;
    let data = bitcode::decode::<D>(&bytes).map_err(|_| {
        RemoteAccessError::Cache(cacache::Error::EntryNotFound(
            db.cache_dir.clone(),
            (&key).to_string(),
        ))
    })?;
    Ok(data)
}
#[derive(Encode, Decode)]
pub struct ObjectCache {
    content_type: String,
    body: Vec<u8>,
    expiry: u128,
}

impl ObjectCache {
    pub fn has_expired(&self) -> bool {
        let duration = Duration::from_millis(self.expiry.try_into().unwrap());
        SystemTime::UNIX_EPOCH
            .checked_add(duration)
            .unwrap()
            .elapsed()
            .is_err()
    }
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
            expiry: SystemTime::now()
                .checked_add(Duration::from_days(1))
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }
}
impl From<ObjectCache> for Response<Vec<u8>> {
    fn from(value: ObjectCache) -> Self {
        let resp_builder = ResponseBuilder::new().header(CONTENT_TYPE, value.content_type);
        resp_builder.body(value.body).unwrap()
    }
}
impl From<&ObjectCache> for Response<Vec<u8>> {
    fn from(value: &ObjectCache) -> Self {
        let resp_builder = ResponseBuilder::new().header(CONTENT_TYPE, value.content_type.clone());
        resp_builder.body(value.body.clone()).unwrap()
    }
}
