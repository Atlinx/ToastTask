use reqwest::header::AUTHORIZATION;
use rocket::{
    outcome::{try_outcome, Outcome},
    request::{self, FromRequest},
    Request,
};
use rocket_db_pools::Connection;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

use crate::{
    database::BackendDb,
    models::{session::SessionModel, user::UserModel},
    responses::{guard_unauthorized, MapReqAPIResponse},
    utils::ResultAsOutcome,
};

#[derive(Debug)]
pub struct Auth<T>(pub T);

impl<T> Deref for Auth<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Auth<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<UserModel> {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = req.headers().get(AUTHORIZATION.as_str()).collect();
        if keys.len() != 1 {
            return guard_unauthorized(req, "Missing authorization header.");
        }

        let auth_header = keys[0];
        let session_id: Uuid = try_outcome!(Uuid::parse_str(&auth_header.replace("Bearer ", ""))
            .as_outcome()
            .map_unauthorized(req, "Bearer session token must be valid UUID."));

        let mut db = try_outcome!(req
            .guard::<Connection<BackendDb>>()
            .await
            .map_internal_server_error(req, "Could not fetch database."));

        let session: SessionModel = try_outcome!(sqlx::query_as!(
            SessionModel,
            "SELECT * FROM sessions WHERE id = $1",
            session_id
        )
        .fetch_one(&mut *db)
        .await
        .as_outcome()
        .map_unauthorized(req, "Invalid session token."));

        // Prune sessions
        try_outcome!(sqlx::query_as!(
            SessionModel,
            "DELETE FROM sessions WHERE user_id = $1 AND CURRENT_TIMESTAMP >= expire_at",
            session.user_id
        )
        .execute(&mut *db)
        .await
        .as_outcome()
        .map_internal_server_error(req, "Pruning expired sessions failed."));

        let user: UserModel = try_outcome!(sqlx::query_as!(
            UserModel,
            "SELECT * FROM users WHERE id = $1",
            session.user_id
        )
        .fetch_one(&mut *db)
        .await
        .as_outcome()
        .map_internal_server_error(req, "Session points to invalid user."));

        Outcome::Success(Auth(user))
    }
}
