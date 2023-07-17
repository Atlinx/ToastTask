use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct DiscordUserLoginModel {
    pub user_id: Uuid,
    pub client_id: String,
}
