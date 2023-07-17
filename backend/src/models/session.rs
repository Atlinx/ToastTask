use chrono::NaiveDateTime;
use ipnetwork::IpNetwork;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct SessionModel {
    pub id: Uuid,
    pub ip: IpNetwork,
    pub platform: String,
    pub user_agent: String,
    pub created_at: NaiveDateTime,
    pub expire_at: NaiveDateTime,
    pub user_id: Uuid,
}
