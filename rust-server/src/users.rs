use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};
use argon2::{
    password_hash::{rand_core, PasswordHash, PasswordHasher, SaltString},
    Argon2,
};

use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};

#[derive(Serialize, Deserialize)]
struct CreateUserRq {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct UserEntry {
    email: String,
    hash: String,
    salt: String,
}

pub fn create_user(body: String, db: Arc<impl Database>) -> Result<String, HttpError> {
    let body: users::CreateUserRq = if let Ok(valid_body) = serde_json::from_str(&body) {
        valid_body
    } else {
        return Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "Request body has invalid format.".to_string(), // Todo, make this json
        });
    };

    if let Some(_) = db.get(&body.username) {
        // User already exists in the db
        return Err(HttpError {
            code: HttpErrorCode::Error409Conflict,
            message: "User already exists.".to_string(), // Todo, make this json
        });
    }

    // Reference https://docs.rs/argon2/latest/argon2/
    let salt: SaltString = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(&body.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user_entry = UserEntry {
        email: body.email,
        hash,
        salt: salt.to_string(), // Js9 - not sure this is right way to store salt
    };

    db.set(body.username, serde_json::to_string(&user_entry).unwrap());

    Ok("token".to_string())
}
