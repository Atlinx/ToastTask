use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct EmailUserLogin {
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    pub password: String,
}
