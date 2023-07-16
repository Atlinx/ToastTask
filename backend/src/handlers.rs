use crate::database::BackendDb;
use crate::models::user::UserModel;
use crate::responses::{
    bad_request, forbidden, internal_server_error, not_found, service_unavailable, unauthorized,
    APIResponse,
};
use rocket::http::Status;
use rocket::outcome::{try_outcome, Outcome};
use rocket::request::{self, FromRequest, Request};
use rocket::{catch, Build, Rocket};
use rocket_db_pools::Connection;

#[catch(400)]
fn bad_request_handler() -> APIResponse {
    bad_request()
}

#[catch(401)]
fn unauthorized_handler() -> APIResponse {
    unauthorized()
}

#[catch(403)]
fn forbidden_handler() -> APIResponse {
    forbidden()
}

#[catch(404)]
fn not_found_handler() -> APIResponse {
    not_found()
}

#[catch(500)]
fn internal_server_error_handler() -> APIResponse {
    internal_server_error()
}

#[catch(503)]
fn service_unavailable_handler() -> APIResponse {
    service_unavailable()
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
    rocket.register(
        "/",
        catchers![
            bad_request_handler,
            unauthorized_handler,
            forbidden_handler,
            not_found_handler,
            internal_server_error_handler,
            service_unavailable_handler,
        ],
    )
}
