use std::{
    fmt::Display,
    ops::{FromResidual, Try},
};

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

impl<T: Serialize, D: Display> Try for UserValue<T, D> {
    type Output = T;

    type Residual = D;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            UserValue::Ok(data) => std::ops::ControlFlow::Continue(data),
            UserValue::Err(e) => std::ops::ControlFlow::Break(e),
        }
    }
}
impl<T: Serialize, D: Display> FromResidual for UserValue<T, D> {
    fn from_residual(residual: <Self as std::ops::Try>::Residual) -> Self {
        UserValue::Err(residual)
    }
}
impl<T: Serialize, D: Display, U> FromResidual<Result<U, D>> for UserValue<T, D> {
    fn from_residual(residual: Result<U, D>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(e) => UserValue::Err(e),
        }
    }
}

