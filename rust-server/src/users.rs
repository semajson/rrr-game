use crate::{users, Database};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{fs, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct CreateUserRq {
    username: String,
    email: String,
    password: String,
}

pub fn create_user(createUserRq: CreateUserRq, db: Arc<impl Database>) -> bool {
    if let Some(_) = db.get(&createUserRq.username) {
        return false;
    }

    true
}
