mod common;

use axum::http::{StatusCode, header};
use serde_json::json;

#[tokio::test]
async fn register_success() {
    let server = common::setup().await;
    let response = server
        .post("/api/v1/auth/register")
        .json(&json!({ "username": "alice", "password": "password123" }))
        .await;
    println!("register_success → {}", response.text());
    response.assert_status_ok();
    response.assert_json_contains(&json!({ "username": "alice" }));
}

#[tokio::test]
async fn register_duplicate_username() {
    let server = common::setup().await;
    let body = json!({ "username": "alice", "password": "password123" });
    server.post("/api/v1/auth/register").json(&body).await;
    let response = server.post("/api/v1/auth/register").json(&body).await;
    println!("register_duplicate → {}", response.text());
    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_success() {
    let server = common::setup().await;
    server
        .post("/api/v1/auth/register")
        .json(&json!({ "username": "bob", "password": "password123" }))
        .await;

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({ "username": "bob", "password": "password123" }))
        .await;
    println!("login_success → {}", response.text());
    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[tokio::test]
async fn login_wrong_password() {
    let server = common::setup().await;
    server
        .post("/api/v1/auth/register")
        .json(&json!({ "username": "carol", "password": "password123" }))
        .await;

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({ "username": "carol", "password": "wrongpassword" }))
        .await;
    println!("login_wrong_password → {}", response.text());
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_unknown_user() {
    let server = common::setup().await;
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({ "username": "nobody", "password": "password123" }))
        .await;
    println!("login_unknown_user → {}", response.text());
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn refresh_token() {
    let server = common::setup().await;
    server
        .post("/api/v1/auth/register")
        .json(&json!({ "username": "dave", "password": "password123" }))
        .await;

    let login = server
        .post("/api/v1/auth/login")
        .json(&json!({ "username": "dave", "password": "password123" }))
        .await
        .json::<serde_json::Value>();

    let response = server
        .post("/api/v1/auth/refresh")
        .json(&json!({ "refresh_token": login["refresh_token"] }))
        .await;
    println!("refresh_token → {}", response.text());
    response.assert_status_ok();
    assert!(response.json::<serde_json::Value>()["access_token"].is_string());
}

#[tokio::test]
async fn logout() {
    let server = common::setup().await;
    server
        .post("/api/v1/auth/register")
        .json(&json!({ "username": "eve", "password": "password123" }))
        .await;

    let login = server
        .post("/api/v1/auth/login")
        .json(&json!({ "username": "eve", "password": "password123" }))
        .await
        .json::<serde_json::Value>();

    let access_token = login["access_token"].as_str().unwrap();
    let refresh_token = login["refresh_token"].as_str().unwrap();

    let response = server
        .post("/api/v1/auth/logout")
        .add_header(header::AUTHORIZATION, common::bearer(access_token))
        .json(&json!({ "refresh_token": refresh_token }))
        .await;
    println!("logout → status {}", response.status_code());
    response.assert_status(StatusCode::NO_CONTENT);
}
