use std::env;

use rocket::{Build, Rocket};

pub struct Config {
    base_url: String,
    backend_port: u16,
    web_port: u16,
}

pub trait MountConfig {
    fn mount_config(self) -> Self;
}

impl Config {
    pub fn backend_url(&self) -> String {
        format!("{}:{}", self.base_url, self.backend_port)
    }
    pub fn web_url(&self) -> String {
        format!("{}:{}", self.base_url, self.web_port)
    }
}

impl MountConfig for Rocket<Build> {
    fn mount_config(self) -> Self {
        self.manage(Config {
            backend_port: env::var("BACKEND_PORT")
                .expect("BACKEND_PORT must be set")
                .parse::<u16>()
                .expect("BACKEND_PORT must be a u16"),
            web_port: env::var("WEB_PORT")
                .expect("WEB_PORT must be set")
                .parse::<u16>()
                .expect("WEB_PORT must be a u16"),
            base_url: env::var("BASE_URL").expect("BASE_URL must be set"),
        })
    }
}
