#[macro_use]
extern crate rocket;

use config::AppConfig;
use rocket::{Build, Rocket};
use sqlx::postgres::PgConnectOptions;

pub mod api;
pub mod config;
pub mod database;
pub mod guards;
pub mod handlers;
pub mod macros;
pub mod models;
pub mod responses;
pub mod utils;
pub mod validation;

pub struct RocketBuilderConfig {
    pub db_conn_options: Option<PgConnectOptions>,
}

impl Default for RocketBuilderConfig {
    fn default() -> Self {
        RocketBuilderConfig {
            db_conn_options: None,
        }
    }
}

/// Constructs a new Rocket instance.
///
/// This function takes care of attaching all routes and handlers of the application.
pub fn create_rocket_default(config_name: &str) -> Result<Rocket<Build>, String> {
    let app_config = config::get_config(config_name).map_err(|x| format!("{}", x))?;
    create_rocket(&app_config)
}

/// Constructs a new Rocket instance.
///
/// This function takes care of attaching all routes and handlers of the application.
///
/// Takes a RocketBuilderConfig that can
/// overwrite some configuration settings.
pub fn create_rocket(app_config: &AppConfig) -> Result<Rocket<Build>, String> {
    let mut rocket = rocket::custom(app_config.to_rocket_figment()).manage(app_config.clone());
    rocket = api::mount_rocket(rocket);
    rocket = database::mount_rocket(rocket);
    rocket = handlers::mount_rocket(rocket);
    Ok(rocket)
}
