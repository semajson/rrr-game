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
    let rq: users::CreateUserRq = serde_json::from_str(&body).unwrap();

    if let Some(_) = db.get(&rq.username) {
        // User already exists in the db
        return Err(HttpError {
            code: HttpErrorCode::Error409Conflict,
            message: "User already exists in db".to_string(),
        });
    }

    // Reference https://docs.rs/argon2/latest/argon2/
    let salt: SaltString = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(&rq.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user_entry = UserEntry {
        email: rq.email,
        hash,
        salt: salt.to_string(), // Js9 - not sure this is right way to store salt
    };

    db.set(rq.username, serde_json::to_string(&user_entry).unwrap());

    Ok("token".to_string())
}
