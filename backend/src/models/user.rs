use rocket_db_pools::database::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::BackendDb;

pub struct UserModel {
    pub id: Uuid,
    pub username: String,
}

impl UserModel {
    pub fn get_user_from_session(session_token: &str, db_conn: &Connection<BackendDb>) {
        // sqlx::query!("SELECT * FROM sessions WHERE")
        // session_token
        // TODO:
        todo!();
    }
}
