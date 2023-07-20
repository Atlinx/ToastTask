use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct LabelModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct TaskLabelModel {
    pub task_id: Uuid,
    pub label_id: Uuid,
}
