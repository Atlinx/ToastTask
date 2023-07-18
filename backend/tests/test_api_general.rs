use parking_lot::Mutex;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::error::Error;
use toast_task::{create_rocket, RocketBuilderConfig};

mod commons;

static DB_LOCK: Mutex<()> = Mutex::new(());

#[sqlx::test]
async fn index(_: PgPoolOptions, pg_conn_options: PgConnectOptions) -> Result<(), Box<dyn Error>> {
    let client = commons::create_client(pg_conn_options);
    let req = client.get("/").send().await.expect("Expected response.");
    let body_text = req.text().await.expect("Expected text response");
    assert_eq!(body_text, "Toast APId 🍞");
    Ok(())
}

// speculate! {
//   before {
//     dotenv::dotenv().ok();
//     let _lock = DB_LOCK.lock(); // Mutex automatically unlocked since Rust knows when lifecycle ends
//     let rocket = rocket_factory("testing").unwrap();
//     let client = Client::tracked(rocket).unwrap();
//   }
//   describe "index" {
//     it "displays the title of the backend" {

//     }
//   }
// }
