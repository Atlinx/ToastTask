use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct DiscordUserLoginModel {
    pub user_id: Uuid,
    pub client_id: String,
}
