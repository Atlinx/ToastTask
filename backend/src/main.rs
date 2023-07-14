#[macro_use]
extern crate rocket;

use std::env;

use dotenv::dotenv;
use rocket::{
    figment::{
        map,
        value::{Map, Value},
    },
    http::Status,
};
use rocket_sync_db_pools::{database, diesel};

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[get("/healthcheck")]
fn healthcheck() -> Status {
    Status::Ok
}

#[catch(503)]
fn service_not_available() -> &'static str {
    "Service not available..."
}

#[database("backend_db")]
struct BackendDbConn(diesel::PgConnection);

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    println!("Using url {}", db_url);
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };
    let figment = rocket::Config::figment().merge(("databases", map!["backend_db" => db]));

    rocket::custom(figment)
        .attach(BackendDbConn::fairing())
        .register("/", catchers![service_not_available])
        .mount("/", routes![index, healthcheck])
}
