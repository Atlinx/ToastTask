use rocket::local::asynchronous::Client;
use sqlx::postgres::PgConnectOptions;
use toast_task::{config::get_config, create_rocket};

pub async fn create_client(pg_conn_options: PgConnectOptions) -> Client {
    let mut app_config = get_config("test").expect("Expected test config to exist.");

    let database = pg_conn_options.get_database().unwrap();
    let postgres_url_no_db = &app_config.database_url[0..app_config
        .database_url
        .rfind("/")
        .unwrap_or(app_config.database_url.len())];
    let database_url = format!("{}/{}", postgres_url_no_db, database);
    app_config.database_url = database_url;

    let rocket = create_rocket(&app_config).expect("Expected rocket to be created.");
    Client::tracked(rocket)
        .await
        .expect("Expected client to be made")
}
