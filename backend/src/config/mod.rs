use rocket::{
    figment::{
        map,
        value::{Map, Value},
        Figment,
    },
    log::LogLevel,
};
use std::{env, fmt};
use time::Duration;

mod dev_config;
mod prod_config;
mod test_config;

#[derive(Clone)]
pub struct AppConfig {
    pub base_url: String,
    pub backend_port: u16,
    pub web_port: u16,
    pub auth_token_timeout_days: Duration,
    pub cors_allow_origin: String,
    pub cors_allow_methods: String,
    pub cors_allow_headers: String,
    pub environment_name: String,
    pub database_url: String,
    pub database_pool_size: u32,
    pub password_salt: String,
    pub session_duration: Duration,
    pub log_level: LogLevel,
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
            database_url: String::from(""),
            database_pool_size: 10,
            password_salt: String::from("default"),
            session_duration: Duration::seconds(10), // TODO: Replace this after testing,
            log_level: LogLevel::Normal,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
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
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            password_salt: env::var("PASSWORD_SALT").expect("PASSWORD_SALT must be set"),
            ..Self::default()
        }
    }

    pub fn backend_url(&self) -> String {
        format!("{}:{}", self.base_url, self.backend_port)
    }
    pub fn web_url(&self) -> String {
        format!("{}:{}", self.base_url, self.web_port)
    }
    pub fn to_rocket_figment(&self) -> Figment {
        let db: Map<_, Value> = map! {
            "url" => self.database_url.clone().into(),
            "pool_size" => self.database_pool_size.into()
        };
        rocket::Config::figment()
            .merge(("port", self.backend_port))
            .merge(("log_level", self.log_level.to_string().to_lowercase()))
            .merge(("databases", map!["backend" => db]))
    }
}

#[derive(Debug)]
pub enum ConfigError {
    /// The environment to fetch a config from is invalid.
    ///
    /// Parameters: (environment_name)
    InvalidEnv(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEnv(string) => write!(f, "{}", string),
        }
    }
}

pub fn get_config(config_env: &str) -> Result<AppConfig, ConfigError> {
    match config_env {
        "production" => Ok(prod_config::config()),
        "development" => Ok(dev_config::config()),
        "test" => Ok(test_config::config()),
        _ => Err(ConfigError::InvalidEnv(format!(
            "No valid config chosen: {}",
            config_env
        ))),
    }
}
