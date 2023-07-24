use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct ListModel {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
}
