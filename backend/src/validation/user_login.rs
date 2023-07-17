use std::collections::HashMap;

use rocket::outcome::try_outcome;
use rocket::{
    data::{self, Data, FromData},
    http::Status,
    serde::json::Json,
    Request,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::responses::APIResponse;
use crate::validation::utils::ToGuardOutcome;

#[derive(Deserialize, Debug, Validate)]
pub struct UserLogin {
    #[serde(skip_deserializing)]
    pub id: Option<Uuid>,
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for UserLogin {
    type Error = Value;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self, Value> {
        use rocket::outcome::Outcome::*;

        let user_login =
            try_outcome!(Json::<UserLogin>::from_data(req, data)
                .await
                .map_failure(|_| {
                    APIResponse::new_message(
                        Status::UnprocessableEntity,
                        "Error while parsing user login.",
                    )
                    .cache_guard_error(req)
                })) as Json<UserLogin>;

        try_outcome!(user_login.validate().to_guard_outcome(req));
        Success(user_login.into_inner())
    }
}
