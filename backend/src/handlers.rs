use crate::database::BackendDb;
use crate::models::user::UserModel;
use crate::responses::{APIResponse, CachedAPIResponse};
use rocket::http::Status;
use rocket::outcome::{try_outcome, Outcome};
use rocket::request::{self, FromRequest, Request};
use rocket::{catch, Build, Rocket};
use rocket_db_pools::Connection;
use rocket_validation::CachedValidationErrors;
use serde_json::json;

#[catch(default)]
fn default_handler(status: Status, req: &Request) -> APIResponse {
    let mut response = req
        .local_cache(|| CachedAPIResponse(APIResponse::from(status)))
        .0
        .clone();
    if let Some(errors) = req.local_cache(|| CachedValidationErrors(None)).0.as_ref() {
        response = response.data(json!({
            "message": Status::BadRequest.to_string(),
            "errors": errors
        }))
    }
    println!(
        "default handler {}, {}",
        status.to_string(),
        Status::ExpectationFailed.to_string()
    );
    response
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserModel {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let db = try_outcome!(request
            .guard::<Connection<BackendDb>>()
            .await
            .map_failure(|f| (f.0, ())));

        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        };

        let auth_header = keys[0];
        let session_token = auth_header.replace("Bearer ", "");

        // TODO:
        todo!();
        // match UserModel::get_user_from_session(&session_token, &*db) {
        //     Some(user) => Outcome::Success(user),
        //     None => Outcome::Failure((Status::Unauthorized, ())),
        // }
    }
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.register("/", catchers![default_handler])
}
