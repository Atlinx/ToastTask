use rocket::http::Status;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

mod commons;

#[sqlx::test]
async fn index(_: PgPoolOptions, pg_conn_options: PgConnectOptions) {
    let client = commons::create_client(pg_conn_options).await;
    let req = client.get("/").dispatch().await;
    assert_eq!(req.status(), Status::Ok);
    let body_text = req.into_string().await.expect("Expected text response");
    assert_eq!(body_text, "Toast API 🍞");
}

#[sqlx::test]
async fn health_check(_: PgPoolOptions, pg_conn_options: PgConnectOptions) {
    let client = commons::create_client(pg_conn_options).await;
    let req = client.get("/healthcheck").dispatch().await;
    assert_eq!(req.status(), Status::Ok);
}
