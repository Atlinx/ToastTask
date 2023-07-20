use argon2rs::argon2i_simple;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserModel {
    pub fn make_password_hash(password: &str, salt: &str) -> Vec<u8> {
        argon2i_simple(password, salt).to_vec()
    }
}
