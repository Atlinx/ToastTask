use crate::responses::{APIResponse, CachedAPIResponse};
use rocket::http::Status;
use rocket::request::Request;
use rocket::{catch, Build, Rocket};
use rocket_validation::CachedValidationErrors;
use serde_json::json;

#[catch(default)]
fn default_handler(status: Status, req: &Request) -> APIResponse {
    let mut response = req
        .local_cache(|| CachedAPIResponse(APIResponse::from(status)))
        .0
        .clone();
    if let Some(errors) = req.local_cache(|| CachedValidationErrors(None)).0.as_ref() {
        response = response.status(Status::BadRequest).data(json!({
            "message": Status::BadRequest.to_string(),
            "errors": errors
        }))
    }
    response
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.register("/", catchers![default_handler])
}
