mod common;

#[tokio::test]
async fn health_ok() {
    let server = common::setup().await;
    let response = server.get("/api/v1/health").await;
    response.assert_status_ok();
    response.assert_json_contains(&serde_json::json!({ "status": "OK" }));
}
