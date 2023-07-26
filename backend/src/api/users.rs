use ipnetwork::IpNetwork;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::{
    database::BackendDb,
    guards::auth::Auth,
    models::{session::SessionModel, user::UserModel},
};

#[get("/me")]
pub async fn get_me(auth_user: Auth<UserModel>, mut db: Connection<BackendDb>) {
    // TODO
    // let user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE ", auth_user.id)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetModel {
    pub id: Uuid,
    pub username: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
    pub discord_login: Option<GetDiscordUserLoginModel>,
    pub email_login: Option<GetEmailUserLoginModel>,
    pub sessions: Vec<GetSessionModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSessionModel {
    pub ip: IpNetwork,
    pub platform: String,
    pub user_agent: String,
    pub created_at: PrimitiveDateTime,
    pub expire_at: PrimitiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDiscordUserLoginModel {
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEmailUserLoginModel {
    pub email: String,
}

pub fn mount_rocket(mut rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/users", routes![get_me])
}
