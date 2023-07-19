use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::commons::http_client::HttpClient;

pub mod discord;
pub mod email;
