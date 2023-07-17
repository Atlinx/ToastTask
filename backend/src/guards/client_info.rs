use std::str::FromStr;

use ipnetwork::IpNetwork;
use reqwest::header::USER_AGENT;
use rocket::{
    outcome::{try_outcome, Outcome},
    request::{self, FromRequest},
    Request,
};

use crate::responses::guard_bad_request;

#[derive(Debug, PartialEq)]
pub enum Platform {
    Web,
    Desktop,
    Mobile,
    Unknown,
}

impl FromStr for Platform {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(Platform::Web),
            "desktop" => Ok(Platform::Desktop),
            "mobile" => Ok(Platform::Mobile),
            _ => Ok(Platform::Unknown),
        }
    }
}

pub struct ClientInfo {
    pub ip: IpNetwork,
    pub platform: Platform,
    pub user_agent: String,
}

// fn get_client_info(cookie_jar: &CookieJar<'_>) -> String {
//   const CLIENT_INFO_COOKIE_NAME: &'static str = "rocket_client_info";
//   match (cookie_jar.get(CLIENT_INFO_COOKIE_NAME)) {
//       Some(cookie) => Platform::from_str(cookie.get_value("platform"),
//       None => String::from("web"),
//   }
// }

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientInfo {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        const CLIENT_PLATFORM_COOKIE_NAME: &'static str = "rocket_client_platform";
        let platform = match req.cookies().get(CLIENT_PLATFORM_COOKIE_NAME) {
            Some(cookie) => Platform::from_str(cookie.value()).unwrap(),
            None => Platform::Unknown,
        };

        let ip_addr = match req
            .client_ip()
            .or_else(|| req.real_ip())
            .or_else(|| Some(req.remote()?.ip()))
        {
            Some(v) => v,
            None => return guard_bad_request(req, "Could not identify ip of client."),
        };
        let ip = IpNetwork::from(ip_addr);

        let user_agent = req
            .headers()
            .get(USER_AGENT.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        Outcome::Success(ClientInfo {
            platform,
            ip,
            user_agent,
        })
    }
}
