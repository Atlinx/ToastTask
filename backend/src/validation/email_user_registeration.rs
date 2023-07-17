use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct EmailUserRegistration {
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    #[validate(length(min = 4, message = "Password must have 4 or more characters."))]
    pub password: String,
    #[validate(length(min = 1, message = "Username must not be empty."))]
    pub username: String,
}
