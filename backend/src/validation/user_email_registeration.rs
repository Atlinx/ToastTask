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
