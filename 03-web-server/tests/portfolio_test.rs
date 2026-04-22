mod common;

use axum::http::{StatusCode, header};
use serde_json::json;

#[tokio::test]
async fn portfolio_requires_auth() {
    let server = common::setup().await;
    let response = server.get("/api/v1/portfolio").await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_portfolio_not_found() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;
    let response = server
        .get("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_portfolio_success() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;
    let response = server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;
    response.assert_status_ok();
    response.assert_json_contains(&json!({ "name": "My Portfolio", "assets": [] }));
}

#[tokio::test]
async fn create_portfolio_duplicate() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;

    server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;

    let response = server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;
    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn add_asset_success() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;

    server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;

    let response = server
        .post("/api/v1/portfolio/asset")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "symbol": "bitcoin", "amount": 0.5 }))
        .await;
    response.assert_status_ok();
    response.assert_json_contains(&json!({ "symbol": "bitcoin", "amount": 0.5 }));
}

#[tokio::test]
async fn add_asset_duplicate() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;

    server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;

    let asset = json!({ "symbol": "bitcoin", "amount": 0.5 });
    server
        .post("/api/v1/portfolio/asset")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&asset)
        .await;

    let response = server
        .post("/api/v1/portfolio/asset")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&asset)
        .await;
    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_portfolio_with_assets() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;

    server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;

    for (symbol, amount) in [("bitcoin", 0.5), ("ethereum", 2.0)] {
        server
            .post("/api/v1/portfolio/asset")
            .add_header(header::AUTHORIZATION, common::bearer(&token))
            .json(&json!({ "symbol": symbol, "amount": amount }))
            .await;
    }

    let response = server
        .get("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["assets"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_asset_success() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;

    server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;

    server
        .post("/api/v1/portfolio/asset")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "symbol": "bitcoin", "amount": 0.5 }))
        .await;

    let response = server
        .delete("/api/v1/portfolio/asset/bitcoin")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .await;
    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_asset_not_found() {
    let server = common::setup().await;
    let token = common::login_as(&server, "alice").await;

    server
        .post("/api/v1/portfolio")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .json(&json!({ "name": "My Portfolio" }))
        .await;

    let response = server
        .delete("/api/v1/portfolio/asset/bitcoin")
        .add_header(header::AUTHORIZATION, common::bearer(&token))
        .await;
    response.assert_status(StatusCode::NOT_FOUND);
}
