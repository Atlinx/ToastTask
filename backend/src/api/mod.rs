use rocket::{Build, Rocket};

pub mod auth;
pub mod general;

pub fn mount_rocket(mut rocket: Rocket<Build>) -> Rocket<Build> {
    rocket = auth::mount_rocket(rocket);
    rocket = general::mount_rocket(rocket);
    rocket
}
