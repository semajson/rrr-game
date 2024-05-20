use crate::{
    jwt,
    requests::{HttpError, HttpErrorCode},
    users, Database,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn create_game(username: String, db: Arc<impl Database>) -> Result<String, HttpError> {
    // Check if user is in a game already
    let user_info = users::get_user_raw(&username, db)?;

    // /?
    Ok("test".to_string())
}
