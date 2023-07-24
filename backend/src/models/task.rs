use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct TaskModel {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub list_id: Uuid,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
    pub due_at: PrimitiveDateTime,
    pub due_text: String,
    pub completed: bool,
    pub title: String,
    pub description: Option<String>,
}
