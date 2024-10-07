use tauri::Error;

use crate::{data::DatabaseInterface, AppAuthenticationStatus, User};

pub fn setup(db: DatabaseInterface) -> Result<(AppAuthenticationStatus, Option<User>), Error> {
    let data = db.get_data(true).unwrap();
    // If we have certs, exit for now
    if data.certs.is_some() {
        // TODO: check if it's still valid, and fetch user information
        return Ok((AppAuthenticationStatus::SignedInNeedsReauth, None));
    }

    return Ok((AppAuthenticationStatus::SignedOut, None));
}
