use super::AppConfig;

pub fn config() -> AppConfig {
    AppConfig {
        backend_port: 7598,
        ..AppConfig::from_env()
    }
}
