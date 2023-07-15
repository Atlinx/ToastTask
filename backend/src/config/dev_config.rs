use super::AppConfig;

pub fn config() -> AppConfig {
    AppConfig {
        ..AppConfig::from_env()
    }
}
