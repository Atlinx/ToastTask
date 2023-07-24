use argon2rs::argon2i_simple;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

impl UserModel {
    pub fn make_password_hash(password: &str, salt: &str) -> Vec<u8> {
        argon2i_simple(password, salt).to_vec()
    }
}
