use rocket::{Build, Rocket};
use serde::Serialize;
use uuid::Uuid;

pub mod discord;
pub mod email;

#[derive(Serialize)]
pub struct SessionPayload {
    pub user_id: Uuid,
    pub session_token: Uuid,
}

impl From<SessionPayload> for serde_json::Value {
    fn from(value: SessionPayload) -> Self {
        serde_json::to_value(value).unwrap()
    }
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    let mut rocket = discord::mount_rocket(rocket);
    rocket = email::mount_rocket(rocket);
    rocket
}
