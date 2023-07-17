use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct LabelModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
}

#[derive(sqlx::FromRow)]
pub struct TaskLabelModel {
    pub task_id: Uuid,
    pub label_id: Uuid,
}
