use rocket::{Build, Rocket};
use rocket_db_pools::Database;

#[derive(Database)]
#[database("backend")]
pub struct BackendDb(sqlx::PgPool);

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.attach(BackendDb::init())
}
