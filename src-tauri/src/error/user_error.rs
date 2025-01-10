use std::fmt::Display;

use serde::Serialize;

pub enum UserValue<T, D>
where
    T: Serialize,
    D: Display,
{
    Ok(T),
    Err(D),
}
impl<T: Serialize, D: Display> Serialize for UserValue<T, D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            UserValue::Ok(data) => data.serialize(serializer),
            UserValue::Err(err) => serializer.serialize_str(err.to_string().as_ref()),
        }
    }
}

impl<T: Serialize, D: Display> From<Result<T, D>> for UserValue<T, D> {
    fn from(value: Result<T, D>) -> Self {
        match value {
            Ok(data) => UserValue::Ok(data),
            Err(data) => UserValue::Err(data),
        }
    }
}
