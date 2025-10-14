use serde::Serialize;

#[derive(Clone, Copy, Serialize, Eq, PartialEq)]
pub enum AppStatus {
    NotConfigured,
    Offline,
    ServerError,
    SignedOut,
    SignedIn,
    SignedInNeedsReauth,
    ServerUnavailable,
}
