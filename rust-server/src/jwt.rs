use std::mem::zeroed;

use serde::{Deserialize, Serialize};

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

use crate::requests::{HttpError, HttpErrorCode};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

pub fn create_jwt(username: &String, secret: &String) -> String {
    let my_claims = Claims {
        sub: username.clone(),
        exp: get_current_timestamp(),
    };

    encode(
        &jsonwebtoken::Header::new(Algorithm::HS512),
        &my_claims,
        &EncodingKey::from_secret(&secret.as_bytes()),
    )
    .unwrap()
}

pub fn validate_jwt(token: &String, secret: &String) -> Result<String, HttpError> {
    // println!("Token is: {:?}", token);

    // Validate token
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(c) => Ok(c.claims.sub),
        Err(_) => Err(HttpError {
            code: HttpErrorCode::Error401Unauthorized,
            message: "Error decoding jwt token".to_string(),
        }),
    }
}