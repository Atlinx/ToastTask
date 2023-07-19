#![cfg(test)]

use crate::commons;
use reqwest::StatusCode;

#[rocket::async_test]
async fn index() {
    let client = commons::setup().await;
    let req = client.get("/").send().await.expect("Expected response");
    assert_eq!(req.status(), StatusCode::OK);
    let body_text = req.text().await.expect("Expected text");
    assert_eq!(body_text, "Toast API 🍞");
}

#[rocket::async_test]
async fn health_check() {
    let client = commons::setup().await;
    let req = client
        .get("/healthcheck")
        .send()
        .await
        .expect("Expected response");
    assert_eq!(req.status(), StatusCode::OK);
}
