use rocket::{Build, Rocket};

pub mod auth;
pub mod general;
pub mod labels;
pub mod lists;
pub mod sessions;
pub mod tasks;
pub mod users;
pub mod utils;

pub fn mount_rocket(mut rocket: Rocket<Build>) -> Rocket<Build> {
    rocket = auth::mount_rocket(rocket);
    rocket = general::mount_rocket(rocket);
    rocket = lists::mount_rocket(rocket);
    rocket = labels::mount_rocket(rocket);
    rocket = tasks::mount_rocket(rocket);
    rocket = sessions::mount_rocket(rocket);
    rocket = users::mount_rocket(rocket);
    rocket
}
