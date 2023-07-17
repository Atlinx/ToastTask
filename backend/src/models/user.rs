use chrono::{DateTime, Utc};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::BackendDb;

#[derive(sqlx::FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub create_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserModel {
    pub fn get_user_from_session(session_token: &str, db_conn: &Connection<BackendDb>) {
        // sqlx::query!("SELECT * FROM sessions WHERE")
        // session_token
        // TODO:
        todo!();
    }
}
