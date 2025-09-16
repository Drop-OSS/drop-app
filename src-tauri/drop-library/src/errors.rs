pub enum DropLibraryError {
    NetworkError(reqwest::Error),
    ServerError(drop_errors::drop_server_error::ServerError),
    Unconfigured,
}

impl From<reqwest::Error> for DropLibraryError {
    fn from(value: reqwest::Error) -> Self {
        DropLibraryError::NetworkError(value)
    }
}