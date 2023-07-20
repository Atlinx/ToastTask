use rocket::{http::Status, serde::json::Json, Build, Rocket, State};
use rocket_db_pools::Connection;
use rocket_validation::Validated;
use sqlx::Acquire;

use crate::{
    config::AppConfig,
    database::BackendDb,
    guards::client_info::ClientInfo,
    models::{email_user_login::EmailUserLoginModel, session::create_session, user::UserModel},
    responses::{bad_request, result_bad_request, APIResponse, APIResult, MapAPIResponse},
    validation::{
        email_user_login::EmailUserLogin, email_user_registeration::EmailUserRegistration,
    },
};

use super::SessionPayload;

#[post(
    "/login/email",
    data = "<email_user_login>",
    format = "application/json"
)]
async fn email_login(
    email_user_login: Validated<Json<EmailUserLogin>>,
    config: &State<AppConfig>,
    mut db: Connection<BackendDb>,
    client_info: ClientInfo,
) -> APIResult {
    let email_user_login = email_user_login.into_deep_inner();
    let email_user_login_data = sqlx::query_as!(
        EmailUserLoginModel,
        "SELECT * FROM email_user_logins WHERE email = $1",
        email_user_login.email
    )
    .fetch_one(&mut *db)
    .await
    .map_unauthorized("Username or password is incorrect.")?;

    if email_user_login_data.password_hash
        != UserModel::make_password_hash(&email_user_login.password, &config.password_salt)
    {
        return result_bad_request("Username or password is incorrect.");
    }

    // Create new session
    let new_session_id =
        create_session(db, config, &client_info, email_user_login_data.user_id).await?;

    Ok(APIResponse::new(
        Status::Ok,
        SessionPayload {
            user_id: email_user_login_data.user_id,
            session_token: new_session_id,
        }
        .into(),
    ))
}

#[post(
    "/register/email",
    data = "<email_user_registration>",
    format = "application/json"
)]
async fn email_registeration(
    email_user_registration: Validated<Json<EmailUserRegistration>>,
    config: &State<AppConfig>,
    mut db: Connection<BackendDb>,
) -> APIResult {
    let email_user_registration = email_user_registration.into_deep_inner();
    let existing_email_login = sqlx::query_as!(
        EmailUserLoginModel,
        "SELECT * FROM email_user_logins WHERE email = $1",
        email_user_registration.email
    )
    .fetch_optional(&mut *db)
    .await
    .map_internal_server_error("Error accessing database.")?;

    if !existing_email_login.is_none() {
        return Err(bad_request("Email is already taken."));
    }

    let mut trans = db.begin().await.map_err(|_| {
        APIResponse::new_message(
            Status::InternalServerError,
            "Create user transaction failed to start.",
        )
    })?;
    let new_user_id = sqlx::query!(
        "INSERT INTO users(username) VALUES ($1) RETURNING id",
        email_user_registration.username
    )
    .fetch_one(&mut trans)
    .await
    .map_internal_server_error("Failed to create new user.")?
    .id;

    let hashed_password =
        UserModel::make_password_hash(&email_user_registration.password, &config.password_salt);
    sqlx::query!(
        "INSERT INTO email_user_logins(user_id, email, password_hash) VALUES ($1, $2, $3)",
        new_user_id,
        email_user_registration.email,
        hashed_password,
    )
    .execute(&mut trans)
    .await
    .map_internal_server_error("Failed to create email login.")?;

    trans
        .commit()
        .await
        .map_internal_server_error("Failed to commit create user transaction")?;
    Ok(APIResponse::new_message(
        Status::Ok,
        &format!("Email registration worked! {:?}", email_user_registration),
    ))
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![email_login, email_registeration])
}
