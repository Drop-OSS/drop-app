use cacache::Integrity;
use openssl::hash::{hash, MessageDigest};
use rustix::path::Arg;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

use crate::{database::db::borrow_db_checked, error::remote_access_error::RemoteAccessError};


#[macro_export]
macro_rules! offline {
    ($var:expr, $func1:expr, $func2:expr, $( $arg:expr ),* ) => {

        if crate::borrow_db_checked().settings.force_offline || $var.lock().unwrap().status == crate::AppStatus::Offline {
            $func2( $( $arg ), *)
        } else {
            $func1( $( $arg ), *)
        }
    }
}

pub fn cache_object<'a, K: AsRef<str>, D: Serialize + DeserializeOwned>(key: K, data: &D) -> Result<Integrity, RemoteAccessError> {
    let bytes = serde_json::to_vec(data).unwrap();
    cacache::write_sync(&borrow_db_checked().cache_dir, key, bytes).map_err(|e| RemoteAccessError::Cache(e))
}
pub fn get_cached_object<'a, K: AsRef<str>, D: Serialize + DeserializeOwned>(key: K) -> Result<D, RemoteAccessError> {
    let bytes = cacache::read_sync(&borrow_db_checked().cache_dir, key).map_err(|e| RemoteAccessError::Cache(e))?;
    let data = serde_json::from_slice::<D>(&bytes).unwrap();
    Ok(data)
}