use chrono::{NaiveDateTime, Utc};
use ipnetwork::IpNetwork;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    database::BackendDb,
    guards::client_info::ClientInfo,
    responses::{APIResponse, MapAPIResponse},
};

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: Uuid,
    pub ip: IpNetwork,
    pub platform: String,
    pub user_agent: String,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub expire_at: NaiveDateTime,
}

/// Creates a session in the datatbase and return it's id.
pub async fn create_session(
    mut db: Connection<BackendDb>,
    config: &AppConfig,
    client_info: &ClientInfo,
    user_id: Uuid,
) -> Result<Uuid, APIResponse> {
    let now = Utc::now().naive_utc();
    let created_at = now;
    let expire_at = now + config.session_duration;
    let result = sqlx::query!("INSERT INTO sessions (ip, platform, user_agent, created_at, expire_at, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id", client_info.ip, client_info.platform.to_string(), client_info.user_agent, created_at, expire_at, user_id).fetch_one(&mut *db).await.map_internal_server_error("Failed to create session.")?;
    Ok(result.id)
}
