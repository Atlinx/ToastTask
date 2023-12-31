use crate::{guards::auth::Auth, models::user::UserModel, responses::APIResponse};
use rocket::{http::Status, routes, Build, Rocket};
use serde_json::json;

#[get("/")]
fn index() -> &'static str {
    "Toast API 🍞"
}

#[get("/healthcheck")]
fn healthcheck() -> Status {
    Status::Ok
}

#[get("/whoami")]
pub fn whoami(current_user: Auth<UserModel>) -> APIResponse {
    APIResponse::new(Status::Ok, json!(current_user.username))
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .manage(reqwest::Client::new())
        .mount("/", routes![index, healthcheck])
}
