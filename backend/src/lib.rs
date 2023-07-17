#[macro_use]
extern crate rocket;

use rocket::Build;

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

/// Constructs a new Rocket instance.
///
/// This function takes care of attaching all routes and handlers of the application.
pub fn rocket_factory(config_name: &str) -> Result<rocket::Rocket<Build>, String> {
    let app_config = config::get_config(config_name).map_err(|x| format!("{}", x))?;
    let mut rocket = rocket::custom(app_config.to_rocket_figment()).manage(app_config);
    rocket = api::mount_rocket(rocket);
    rocket = database::mount_rocket(rocket);
    rocket = handlers::mount_rocket(rocket);
    Ok(rocket)
}
