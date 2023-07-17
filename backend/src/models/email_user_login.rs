use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct EmailUserLoginModel {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: Vec<u8>,
}
