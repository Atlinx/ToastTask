#[macro_use]
extern crate rocket;

use auth::MountAuth;
use config::MountConfig;
use dotenv::dotenv;
use rocket::{
    figment::{
        map,
        value::{Map, Value},
        Figment,
    },
    http::Status,
};
use rocket_db_pools::diesel::PgPool;
use rocket_db_pools::Database;
use std::env;

mod auth;
mod config;
mod macros;
mod models;
mod schema;

#[get("/")]
fn index() -> &'static str {
    "Toast API 🍞"
}

#[get("/healthcheck")]
fn healthcheck() -> Status {
    Status::Ok
}

#[derive(Database)]
#[database("backend")]
pub struct BackendDb(PgPool);

fn customize_config(figment: Figment) -> Figment {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };
    figment.merge(("databases", map!["backend" => db]))
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    rocket::custom(customize_config(rocket::Config::figment()))
        .attach(BackendDb::init())
        .mount("/", routes![index, healthcheck])
        .manage(reqwest::Client::new())
        .mount_auth()
        .mount_config()
}
