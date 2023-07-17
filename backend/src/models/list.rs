use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct ListModelModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
}

#[derive(sqlx::FromRow)]
pub struct ListRelationModel {
    pub child_list_id: Uuid,
    pub parent_list_id: Uuid,
}
