use dotenv::dotenv;
use std::env;

#[rocket::main]
async fn main() -> Result<(), String> {
    dotenv().ok();

    let config_name = env::var("CONFIG_ENV").expect("CONFIG_ENV must be set");
    let rocket = toast_task::create_rocket_default(&config_name)?;
    let _ = rocket.launch().await;
    Ok(())
}
