use crate::{
    requests::{HttpError, HttpErrorCode},
    users, Database,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
            message: "Request body has invalid format.".to_string(),
        });
    };

    if db.get(&body.username).is_some() {
        // User already exists in the db
        return Err(HttpError {
            code: HttpErrorCode::Error409Conflict,
            message: "User already exists.".to_string(),
        });
    }

    // Reference https://docs.rs/argon2/latest/argon2/
    let salt: SaltString = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
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

#[derive(Serialize, Deserialize)]
struct LoginRq {
    username: String,
    password: String,
}

pub fn login(body: String, db: Arc<impl Database>) -> Result<String, HttpError> {
    let body: users::LoginRq = if let Ok(valid_body) = serde_json::from_str(&body) {
        valid_body
    } else {
        return Err(HttpError {
            code: HttpErrorCode::Error400BadRequest,
            message: "Request body has invalid format.".to_string(),
        });
    };

    // Get user info
    let user_info: UserEntry = if let Some(user_info) = db.get(&body.username) {
        serde_json::from_str(&user_info).unwrap()
    } else {
        // User doesn't exist
        // Todo - should this just be a generic error in order to not leak info?
        return Err(HttpError {
            code: HttpErrorCode::Error401Unauthorized,
            message: "User doesn't exist".to_string(),
        });
    };

    // Check password
    let parsed_hash = PasswordHash::new(&user_info.hash).unwrap();
    if Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        Ok("success".to_string())
    } else {
        // Password incorrect
        // Todo - should this just be a generic error in order to not leak info?
        Err(HttpError {
            code: HttpErrorCode::Error401Unauthorized,
            message: "Password incorrect".to_string(),
        })
    }
}
