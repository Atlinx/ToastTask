use rocket::log::LogLevel;

use super::AppConfig;

pub fn config() -> AppConfig {
    AppConfig {
        log_level: LogLevel::Off,
        ..AppConfig::from_env()
    }
}
