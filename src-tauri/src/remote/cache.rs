use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{
    database::{db::borrow_db_checked, models::data::Database},
    error::remote_access_error::RemoteAccessError,
};
use bitcode::{Decode, DecodeOwned, Encode};
use http::{Response, header::CONTENT_TYPE, response::Builder as ResponseBuilder};

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

fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn get_cache_path(base: &Path, key: &str) -> PathBuf {
    let key_hash = hex::encode(md5::compute(key.as_bytes()).0);
    base.join(key_hash)
}

fn write_sync(base: &Path, key: &str, data: Vec<u8>) -> io::Result<()> {
    let cache_path = get_cache_path(base, key);
    let mut file = File::create(cache_path)?;
    file.write_all(&data)?;
    Ok(())
}

fn read_sync(base: &Path, key: &str) -> io::Result<Vec<u8>> {
    let cache_path = get_cache_path(base, key);
    let file = std::fs::read(cache_path)?;
    Ok(file)
}

pub fn cache_object<D: Encode>(key: &str, data: &D) -> Result<(), RemoteAccessError> {
    cache_object_db(key, data, &borrow_db_checked())
}
pub fn cache_object_db<D: Encode>(
    key: &str,
    data: &D,
    database: &Database,
) -> Result<(), RemoteAccessError> {
    let bytes = bitcode::encode(data);
    write_sync(&database.cache_dir, key, bytes).map_err(RemoteAccessError::Cache)
}
pub fn get_cached_object<D: Encode + DecodeOwned>(key: &str) -> Result<D, RemoteAccessError> {
    get_cached_object_db::<D>(key, &borrow_db_checked())
}
pub fn get_cached_object_db<D: DecodeOwned>(
    key: &str,
    db: &Database,
) -> Result<D, RemoteAccessError> {
    let bytes = read_sync(&db.cache_dir, key).map_err(RemoteAccessError::Cache)?;
    let data =
        bitcode::decode::<D>(&bytes).map_err(|e| RemoteAccessError::Cache(io::Error::other(e)))?;
    Ok(data)
}
#[derive(Encode, Decode)]
pub struct ObjectCache {
    content_type: String,
    body: Vec<u8>,
    expiry: u64,
}

impl ObjectCache {
    pub fn has_expired(&self) -> bool {
        let current = get_sys_time_in_secs();
        self.expiry < current
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
            expiry: get_sys_time_in_secs() + 60 * 60 * 24,
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
