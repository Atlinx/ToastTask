use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;
use rocket::http::Status;

#[derive(Deserialize, Debug, Validate)]
pub struct UserLogin<'r> {
    #[serde(skip_deserializing)]
    pub id: Option<Uuid>,
    #[validate(email)]
    pub email: &'r str,
    pub password: &'r str,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for UserLogin<'r> {
    type Error = Value;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        // use rocket::outcome::Outcome::*;
        // use rocket::Error::*;

        let mut data_string = String::new();
        if data.open().read_to_string(&mut data_string).await.is_err() {
          return Failure({
            Status::InternalServerError,
            json!({"_schema": "Internal server error."})
          });
        }
        Success(())
    }
}
