use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn create_game(username: String, db: Arc<impl Database>) -> Result<String, HttpError> {
    // Check if user is in a game already
    let curr_game_id = users::get_user_curr_game_id(&username, db, "rrr-game")?;

    if curr_game_id.is_some() {
        return Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "User is already in a game".to_string(),
        });
    }

    // /?
    Ok("test".to_string())
}
