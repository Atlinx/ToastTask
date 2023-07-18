use std::str::FromStr;

use reqwest::header::HeaderMap;
use rocket::{
    tokio::{self, net::TcpListener},
    Shutdown,
};
use sqlx::{
    migrate::MigrateDatabase,
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Connection, Executor, PgConnection,
};
use toast_task::{config::get_config, create_rocket};
use uuid::Uuid;

use self::http_client::HttpClient;

pub mod http_client;

pub struct ToastTaskApp {
    db_name: String,
    conn_url_no_db: String,
    shutdown: Shutdown,
}

impl ToastTaskApp {
    pub async fn shutdown(self) {
        self.shutdown.notify();
        let postgres_db_conn_url = format!("{}/postgres", &self.conn_url_no_db);
        let mut connection = PgConnection::connect(&postgres_db_conn_url)
            .await
            .expect("Failed to connect to database");
        connection
            .execute(format!("DROP DATABASE \"{}\" WITH (FORCE)", self.db_name).as_str())
            .await
            .expect("Failed to drop database");
    }
}

pub async fn setup() -> (HttpClient, ToastTaskApp) {
    let mut app_config = get_config("test").expect("Expected test config to exist");
    app_config.backend_port = get_next_available_port().await;

    let conn_url_no_db = String::from(
        &app_config.database_url[0..app_config
            .database_url
            .rfind("/")
            .unwrap_or(app_config.database_url.len())],
    );
    let database = Uuid::new_v4().to_string();
    let database_url = format!("{}/{}", conn_url_no_db, database);

    sqlx::Postgres::create_database(&database_url)
        .await
        .expect("Expected database to be created");

    let conn_options = PgConnectOptions::from_str(&database_url)
        .expect("Expected database_url to be valid")
        .disable_statement_logging()
        .clone();
    let connection_pool = PgPoolOptions::new()
        .connect_with(conn_options)
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    app_config.database_url = database_url.clone();

    let rocket = create_rocket(&app_config)
        .expect("Failed to create rocket client")
        .ignite()
        .await
        .expect("Failed to ignite rocket client");

    let app = ToastTaskApp {
        db_name: database,
        conn_url_no_db,
        shutdown: rocket.shutdown(),
    };
    let _ = tokio::spawn(rocket.launch());

    let http_client = HttpClient::new(&app_config.backend_url(), HeaderMap::new())
        .expect("Failed to create HttpClient");

    while let Err(_) = http_client.get("/").send().await {}

    (http_client, app)
}

async fn get_next_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Expected port to be available");
    listener
        .local_addr()
        .expect("Expected local_addr to exist")
        .port()
}
