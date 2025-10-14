use std::fmt::Display;

use serde_with::SerializeDisplay;

#[derive(SerializeDisplay)]
pub enum LibraryError {
    MetaNotFound(String),
    VersionNotFound(String),
}
impl Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LibraryError::MetaNotFound(id) => {
                    format!(
                        "Could not locate any installed version of game ID {id} in the database"
                    )
                }
                LibraryError::VersionNotFound(game_id) => {
                    format!(
                        "Could not locate any installed version  for game id {game_id} in the database"
                    )
                }
            }
        )
    }
}
