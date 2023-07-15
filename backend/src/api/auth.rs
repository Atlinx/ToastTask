use std::error::Error;

use crate::{BackendDb, schema::sessions};

use super::config::Config;
use reqwest::header::AUTHORIZATION;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    response::{Redirect, content},
    Build, Rocket, State,
};
use rocket_db_pools::Connection;
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

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

#[get("/login/session/<session_id>")]
async fn session_login(session_id: &str, mut db: Connection<BackendDb>) -> Status {
    let result = sessions::table.insert_into(table)
    Status::Ok
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
    config: &State<Config>,
    mut db: BackendDb
) -> Result<content::RawHtml<&'static str>, Status> {
    let result = async {
        println!("Starting request");
        let resp = reqwest_client
            .get("https://discordapp.com/api/users/@me")
            .header(
                AUTHORIZATION,
                format!("Bearer {}", token.access_token().to_string()),
            )
            .send()
            .await?;
        println!("Got response");
        let discord_user_resp = resp.json::<DiscordUserResponse>().await?;
        println!(
            "value: {:#?} user: {:#?}",
            token.as_value(),
            discord_user_resp
        );
        cookies.add_private(
            Cookie::build("token", token.access_token().to_string())
                .same_site(SameSite::Lax)
                .finish(),
        );
        Ok::<_, Box<dyn Error>>(
            content::RawHtml(
                format!(r#"<html><head><title>Authenticate</title></head><body></body><script>res = {}; window.opener.postMessage(res, "*");window.close();</script></html>"#, )
            ))
    };
    result.await.or_else(|e| {
        eprintln!("{}: {:?}", crate::name_of!(discord_callback), e.as_ref());
        Err(Status::InternalServerError)
    })
}

pub trait MountAuth {
    fn mount_auth(self) -> Self;
}

impl MountAuth for Rocket<Build> {
    fn mount_auth(self) -> Self {
        let memory_store: MemoryStore<>
        self.attach(OAuth2::<Discord>::fairing("discord"))
            .mount("/", routes![discord_callback, discord_login])
    }
}
