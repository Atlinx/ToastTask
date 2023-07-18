use reqwest::StatusCode;

mod commons;

#[rocket::async_test]
async fn index() {
    let (client, app) = commons::setup().await;
    let req = client.get("/").send().await.expect("Expected response");
    assert_eq!(req.status(), StatusCode::OK);
    let body_text = req.text().await.expect("Expected text");
    assert_eq!(body_text, "Toast API 🍞");
    app.shutdown().await;
}

#[rocket::async_test]
async fn health_check() {
    let (client, app) = commons::setup().await;
    let req = client
        .get("/healthcheck")
        .send()
        .await
        .expect("Expected response");
    assert_eq!(req.status(), StatusCode::OK);
    app.shutdown().await;
}
