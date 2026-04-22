#![allow(dead_code)]

use axum::http::HeaderValue;
use axum_test::TestServer;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use web_server::{migration::Migrator, routes::test_app};

pub async fn setup() -> TestServer {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    TestServer::new(test_app(db))
}

pub fn bearer(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
}

// Реєструє юзера і повертає його access token
pub async fn login_as(server: &TestServer, username: &str) -> String {
    let password = "password123";

    server
        .post("/api/v1/auth/register")
        .json(&serde_json::json!({ "username": username, "password": password }))
        .await;

    let response = server
        .post("/api/v1/auth/login")
        .json(&serde_json::json!({ "username": username, "password": password }))
        .await;

    response.json::<serde_json::Value>()["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}
