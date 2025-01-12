use std::fmt::Display;

pub enum LibraryError {
    MetaNotFound(String),
}
impl Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::MetaNotFound(id) => write!(
                f,
                "Could not locate any installed version of game ID {} in the database",
                id
            ),
        }
    }
}
