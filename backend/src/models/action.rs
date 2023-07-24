use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct ActionModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: PrimitiveDateTime,
    pub action_type: String,
    pub data: serde_json::Value,
}
