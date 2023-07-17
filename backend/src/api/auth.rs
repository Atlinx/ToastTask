use crate::{
    config::AppConfig,
    database::BackendDb,
    models::{email_user_login::EmailUserLoginModel, user::UserModel},
    responses::APIResponse,
    validation::{user_email_registeration::UserEmailRegistration, user_login::UserLogin},
};
use reqwest::header::AUTHORIZATION;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    response::{content, Redirect},
    routes,
    serde::json::Json,
    Build, Rocket, State,
};
use rocket_db_pools::Connection;
use rocket_oauth2::{OAuth2, TokenResponse};
use rocket_validation::Validated;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[post("/login/email", data = "<user_login>")]
async fn email_login(
    user_login: UserLogin,
    mut db: Connection<BackendDb>,
) -> Result<APIResponse, APIResponse> {
    let email_user_login = sqlx::query_as!(
        EmailUserLoginModel,
        "SELECT * FROM email_user_logins WHERE email = $1",
        user_login.email
    )
    .fetch_one(&mut *db)
    .await
    .map_err(|_| {
        APIResponse::new_message(Status::Unauthorized, "Username or password incorrect.")
    })?;

    Ok(APIResponse::new(
        Status::Ok,
        json!({
            "user_id": email_user_login.user_id,
            "token": "hello"
        }),
    ))
}

#[post(
    "/register/email",
    data = "<user_email_registeration>",
    format = "application/json"
)]
async fn email_registeration(
    user_email_registeration: Validated<Json<UserEmailRegistration>>,
    mut db: Connection<BackendDb>,
) -> Result<APIResponse, APIResponse> {
    Ok(APIResponse::new_message(
        Status::Ok,
        &format!("Email registration worked! {:?}", user_email_registeration),
    ))
}

#[get("/login/session/<session_id>")]
async fn session_login(session_id: &str, mut db: Connection<BackendDb>) -> APIResponse {
    // let result = sessions::table.insert_into(table);
    // TODO LATER: FINISH THIS
    todo!()
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
) -> Result<content::RawHtml<&'static str>, Status> {
    // let result = async {
    //     println!("Starting request");
    //     let resp = reqwest_client
    //         .get("https://discordapp.com/api/users/@me")
    //         .header(
    //             AUTHORIZATION,
    //             format!("Bearer {}", token.access_token().to_string()),
    //         )
    //         .send()
    //         .await?;
    //     println!("Got response");
    //     let discord_user_resp = resp.json::<DiscordUserResponse>().await?;
    //     println!(
    //         "value: {:#?} user: {:#?}",
    //         token.as_value(),
    //         discord_user_resp
    //     );
    //     cookies.add_private(
    //         Cookie::build("token", token.access_token().to_string())
    //             .same_site(SameSite::Lax)
    //             .finish(),
    //     );
    //     // TODO LATER: Make sure to return correct value
    //     Ok::<_, Box<dyn Error>>(content::RawHtml(format!(
    //         r#"<html><head><title>Authenticate</title></head><body></body><script>res = {}; window.opener.postMessage(res, "*");window.close();</script></html>"#,
    //         10
    //     )))
    // };
    // result.await.or_else(|e| {
    //     eprintln!("{}: {:?}", name_of!(discord_callback), e.as_ref());
    //     Err(Status::InternalServerError)
    // })
    todo!();
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .attach(OAuth2::<Discord>::fairing("discord"))
        .mount(
            "/",
            routes![
                discord_callback,
                discord_login,
                email_login,
                session_login,
                email_registeration
            ],
        )
        .register("/", catchers![rocket_validation::validation_catcher])
}