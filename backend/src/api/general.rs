use crate::{
    models::user::UserModel,
    responses::{ok, APIResponse},
};
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
pub fn whoami(current_user: UserModel) -> APIResponse {
    ok().data(json!(current_user.username))
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .manage(reqwest::Client::new())
        .mount("/", routes![index, healthcheck])
}
