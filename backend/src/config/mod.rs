use chrono::Duration;
use std::env;

mod dev_config;
mod prod_config;
mod test_config;

pub struct AppConfig {
    pub base_url: String,
    pub backend_port: u16,
    pub web_port: u16,
    pub auth_token_timeout_days: Duration,
    pub cors_allow_origin: String,
    pub cors_allow_methods: String,
    pub cors_allow_headers: String,
    pub environment_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            base_url: String::from("localhost"),
            backend_port: 8000,
            web_port: 8080,
            auth_token_timeout_days: Duration::days(7),
            cors_allow_origin: String::from("*"),
            cors_allow_methods: String::from("*"),
            cors_allow_headers: String::from("*"),
            environment_name: String::from("unconfigured"),
        }
    }
}

impl AppConfig {
    fn from_env() -> Self {
        AppConfig {
            backend_port: env::var("BACKEND_PORT")
                .expect("BACKEND_PORT must be set")
                .parse::<u16>()
                .expect("BACKEND_PORT must be a u16"),
            web_port: env::var("WEB_PORT")
                .expect("WEB_PORT must be set")
                .parse::<u16>()
                .expect("WEB_PORT must be a u16"),
            base_url: env::var("BASE_URL").expect("BASE_URL must be set"),
            ..Self::default()
        }
    }

    pub fn backend_url(&self) -> String {
        format!("{}:{}", self.base_url, self.backend_port)
    }
    pub fn web_url(&self) -> String {
        format!("{}:{}", self.base_url, self.web_port)
    }
}
