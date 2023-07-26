use crate::models::session::SessionModel;
use rocket::{Build, Rocket};

crate::api_get! {
    model_table: "sessions",
    model_type: SessionModel
}

crate::api_delete! {
    model_table: "sessions"
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/sessions", routes![get_single, get_all, delete])
}
