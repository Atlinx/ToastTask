use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct TaskModel {
    pub id: Uuid,
    pub list_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub due_text: String,
    pub completed: bool,
    pub title: String,
    pub description: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct TaskRelationModel {
    pub child_list_id: Uuid,
    pub parent_list_id: Uuid,
}
