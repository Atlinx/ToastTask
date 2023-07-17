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
