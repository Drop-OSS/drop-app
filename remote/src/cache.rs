use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use bitcode::{Decode, DecodeOwned, Encode};
use database::{Database, borrow_db_checked};
use http::{Response, header::CONTENT_TYPE, response::Builder as ResponseBuilder};

use crate::error::{CacheError, RemoteAccessError};

#[macro_export]
macro_rules! offline {
    ($var:expr, $func1:expr, $func2:expr, $( $arg:expr ),* ) => {

        async move {
            if ::database::borrow_db_checked().settings.force_offline
            || $var.lock().status == ::client::app_status::AppStatus::Offline {
            $func2( $( $arg ), *).await
        } else {
            $func1( $( $arg ), *).await
        }
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

fn delete_sync(base: &Path, key: &str) -> io::Result<()> {
    let cache_path = get_cache_path(base, key);
    std::fs::remove_file(cache_path)?;
    Ok(())
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
pub fn clear_cached_object(key: &str) -> Result<(), RemoteAccessError> {
    clear_cached_object_db(key, &borrow_db_checked())
}
pub fn clear_cached_object_db(key: &str, db: &Database) -> Result<(), RemoteAccessError> {
    delete_sync(&db.cache_dir, key).map_err(RemoteAccessError::Cache)?;
    Ok(())
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

impl TryFrom<Response<Vec<u8>>> for ObjectCache {
    type Error = CacheError;

    fn try_from(value: Response<Vec<u8>>) -> Result<Self, Self::Error> {
        Ok(ObjectCache {
            content_type: value
                .headers()
                .get(CONTENT_TYPE)
                .ok_or(CacheError::HeaderNotFound(CONTENT_TYPE))?
                .to_str()
                .map_err(CacheError::ParseError)?
                .to_owned(),
            body: value.body().clone(),
            expiry: get_sys_time_in_secs() + 60 * 60 * 24,
        })
    }
}
impl TryFrom<ObjectCache> for Response<Vec<u8>> {
    type Error = CacheError;
    fn try_from(value: ObjectCache) -> Result<Self, Self::Error> {
        let resp_builder = ResponseBuilder::new().header(CONTENT_TYPE, value.content_type);
        resp_builder
            .body(value.body)
            .map_err(CacheError::ConstructionError)
    }
}
impl TryFrom<&ObjectCache> for Response<Vec<u8>> {
    type Error = CacheError;

    fn try_from(value: &ObjectCache) -> Result<Self, Self::Error> {
        let resp_builder = ResponseBuilder::new().header(CONTENT_TYPE, value.content_type.clone());
        resp_builder
            .body(value.body.clone())
            .map_err(CacheError::ConstructionError)
    }
}
