use serde::{Deserialize, Serialize};

use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Validation,
};

use crate::http::{HttpError, HttpErrorCode};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

pub fn create_jwt(username: &str, secret: &String) -> String {
    let my_claims = Claims {
        sub: username.to_owned(),
        exp: get_current_timestamp(),
    };

    encode(
        &jsonwebtoken::Header::new(Algorithm::HS512),
        &my_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub fn validate_jwt(token: &str, secret: &String) -> Result<String, HttpError> {
    println!("Secret {:?}\n", secret);
    println!("token {:?}\n", token);

    // Validate token
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(c) => Ok(c.claims.sub),
        Err(_) => Err(HttpError {
            code: HttpErrorCode::Error401Unauthorized,
            message: "Error decoding jwt token".to_string(),
        }),
    }
}
