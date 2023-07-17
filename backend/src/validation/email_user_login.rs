use serde::Deserialize;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Debug, Validate)]
pub struct EmailUserLogin {
    #[serde(skip_deserializing)]
    pub id: Option<Uuid>,
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    pub password: String,
}
