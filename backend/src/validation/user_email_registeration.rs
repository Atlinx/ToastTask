use rocket::outcome::try_outcome;
use rocket::{
    data::{self, Data, FromData},
    http::Status,
    request::local_cache,
    serde::json::Json,
    Request,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use crate::responses::APIResponse;
use crate::validation::utils::ToGuardOutcome;

#[derive(Deserialize, Debug, Validate)]
pub struct UserEmailRegistration {
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    #[validate(length(min = 4, message = "Password must have 4 or more characters."))]
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for UserEmailRegistration {
    type Error = Value;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self, Value> {
        use rocket::outcome::Outcome::*;

        let user_email_registration =
            try_outcome!(Json::<UserEmailRegistration>::from_data(req, data)
                .await
                .map_failure(|_| {
                    APIResponse::new_message(
                        Status::UnprocessableEntity,
                        "Error while parsing user login.",
                    )
                    .cache_guard_error(req)
                })) as Json<UserEmailRegistration>;

        try_outcome!(user_email_registration.validate().to_guard_outcome(req));

        Success(user_email_registration.into_inner())
    }
}
