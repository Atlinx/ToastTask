use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAllResponse<T> {
    pub items: Vec<T>,
    pub limit: u32,
    pub page: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostResponse {
    pub id: Uuid,
}
