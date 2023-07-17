use reqwest::header::AUTHORIZATION;
use rocket::{
    http::CookieJar,
    response::{content, Redirect},
    Build, Rocket, State,
};
use rocket_db_pools::Connection;
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    database::BackendDb,
    guards::client_info::ClientInfo,
    models::{discord_user_login::DiscordUserLoginModel, session::create_session},
    responses::{APIResponse, MapAPIResponse},
};

use super::SessionPayload;

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

#[get("/login/discord")]
fn discord_login(oauth: OAuth2<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    oauth.get_redirect(cookies, &["identify"]).unwrap()
}

#[get("/auth/discord")]
async fn discord_callback(
    token: TokenResponse<Discord>,
    reqwest_client: &State<reqwest::Client>,
    config: &State<AppConfig>,
    mut db: Connection<BackendDb>,
    client_info: ClientInfo,
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
    let user_id: Uuid = match sqlx::query_as!(
        DiscordUserLoginModel,
        "SELECT * FROM discord_user_logins WHERE client_id = $1",
        discord_user_resp.id
    )
    .fetch_one(&mut *db)
    .await
    {
        Ok(result) => result.user_id,
        Err(_) => {
            // Create discord login and the user
            let mut trans = db
                .begin()
                .await
                .map_internal_server_error("Create user transaction failed to start")?;

            let new_user_id = sqlx::query!(
                "INSERT INTO users(username) VALUES ($1) RETURNING id",
                discord_user_resp.username
            )
            .fetch_one(&mut trans)
            .await
            .map_internal_server_error("Failed to create new user.")?
            .id;

            sqlx::query!(
                "INSERT INTO discord_user_logins(user_id, client_id) VALUES ($1, $2)",
                new_user_id,
                discord_user_resp.id
            )
            .execute(&mut trans)
            .await
            .map_internal_server_error("Failed to create discord login.")?;

            trans
                .commit()
                .await
                .map_internal_server_error("Failed to commit create user transaction")?;

            new_user_id
        }
    };

    // Make session
    let new_session_id = create_session(db, config, &client_info, user_id).await?;

    let payload: serde_json::Value = SessionPayload {
        user_id,
        session_token: new_session_id,
    }
    .into();

    Ok(content::RawHtml(format!(
        r#"<html><head><title>Authenticate</title></head><body></body><script>res = {}; window.opener.postMessage(res, "*");window.close();</script></html>"#,
        payload.to_string()
    )))
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .attach(OAuth2::<Discord>::fairing("discord"))
        .mount("/", routes![discord_callback, discord_login,])
}
