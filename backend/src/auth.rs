use std::error::Error;

use super::config::Config;
use reqwest::header::AUTHORIZATION;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    response::Redirect,
    Build, Rocket, State,
};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::Deserialize;

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
) -> Result<Redirect, Status> {
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
        Ok::<_, Box<dyn Error>>(Redirect::to(config.web_url()))
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
        self.attach(OAuth2::<Discord>::fairing("discord"))
            .mount("/", routes![discord_callback, discord_login])
    }
}
