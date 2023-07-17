use std::{net::SocketAddr, str::FromStr};

use crate::{
    config::AppConfig,
    database::BackendDb,
    guards::client_info::ClientInfo,
    models::{email_user_login::EmailUserLoginModel, user::UserModel},
    responses::{
        guard_bad_request, result_bad_request, result_internal_server_error, APIResponse,
        MapAPIResponse,
    },
    utils::{socket_addr_to_ip_network, OkAsError},
    validation::{
        email_user_login::EmailUserLogin, email_user_registeration::EmailUserRegistration,
    },
};
use chrono::Utc;
use ipnetwork::IpNetwork;
use reqwest::header::AUTHORIZATION;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    response::{content, Redirect},
    routes,
    serde::json::Json,
    Build, Request, Rocket, State,
};
use rocket_db_pools::Connection;
use rocket_oauth2::{OAuth2, TokenResponse};
use rocket_validation::Validated;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Acquire;
use uuid::Uuid;
use validator::ValidationError;

struct Discord;

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct DiscordUserResponse {
    id: String,
    username: String,
    discriminator: String,
    avatar: String,
    email: Option<String>,
    banner: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    user_id: String,
    session_id: String,
}

#[post(
    "/login/email",
    data = "<email_user_login>",
    format = "application/json"
)]
async fn email_login(
    email_user_login: Validated<Json<EmailUserLogin>>,
    config: &State<AppConfig>,
    mut db: Connection<BackendDb>,
    socket_addr: SocketAddr,
    cookie_jar: &CookieJar<'_>,
    client_info: ClientInfo,
) -> Result<APIResponse, APIResponse> {
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
    let platform = client_info.platform;
    let user_agent = client_info.user_agent;
    let user_ip = socket_addr_to_ip_network(&socket_addr);
    let created_at = Utc::now().naive_utc();
    let expire_at = created_at + config.session_duration;
    let new_session = sqlx::query!("INSERT INTO sessions (ip, platform, user_agent, created_at, expire_at, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id", user_ip, platform, user_agent, created_at, expire_at, email_user_login_data.user_id).fetch_one(&mut *db).await.map_internal_server_error("Failed to create session.")?;

    Ok(APIResponse::new(
        Status::Ok,
        json!({
            "user_id": email_user_login_data.user_id,
            "session_token": new_session.id
        }),
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
) -> Result<APIResponse, APIResponse> {
    let email_user_registration = email_user_registration.into_deep_inner();
    sqlx::query_as!(
        EmailUserLoginModel,
        "SELECT * FROM email_user_logins WHERE email = $1",
        email_user_registration.email
    )
    .fetch_one(&mut *db)
    .await
    .ok_as_err()
    .map_bad_request("Email is already taken.")?;

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

#[get("/login/discord")]
fn discord_login(oauth: OAuth2<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    oauth.get_redirect(cookies, &["identify"]).unwrap()
}

#[get("/auth/discord")]
async fn discord_callback(
    token: TokenResponse<Discord>,
    cookies: &CookieJar<'_>,
    reqwest_client: &State<reqwest::Client>,
    config: &State<AppConfig>,
    mut db: Connection<BackendDb>,
) -> Result<content::RawHtml<String>, APIResponse> {
    let resp = reqwest_client
        .get("https://discordapp.com/api/users/@me")
        .header(
            AUTHORIZATION,
            format!("Bearer {}", token.access_token().to_string()),
        )
        .send()
        .await
        .map_internal_server_error("Failed to fetch Discord identity.")?;

    let discord_user_resp = resp
        .json::<DiscordUserResponse>()
        .await
        .map_internal_server_error("Failed to deserialize Discord identity.")?;

    // Check if discord login exists

    // TODO LATER: Make sure to return correct value
    Ok(content::RawHtml(format!(
        r#"<html><head><title>Authenticate</title></head><body></body><script>res = {}; window.opener.postMessage(res, "*");window.close();</script></html>"#,
        10
    )))
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.attach(OAuth2::<Discord>::fairing("discord")).mount(
        "/",
        routes![
            discord_callback,
            discord_login,
            email_login,
            email_registeration
        ],
    )
}
