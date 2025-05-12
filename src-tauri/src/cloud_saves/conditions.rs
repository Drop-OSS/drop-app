use crate::process::process_manager::Platform;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Condition {
    Os(Platform)
}
