use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct EmailUserLoginModel {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: Vec<u8>,
}
