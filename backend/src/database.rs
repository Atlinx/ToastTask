use rocket_db_pools::{diesel::PgPool, Database};

#[derive(Database)]
#[database("backend")]
pub struct DbConn(PgPool);
