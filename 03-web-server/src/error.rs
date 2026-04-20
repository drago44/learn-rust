use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("upstream unavailable")]
    BadGateway,
    #[error("not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AppError::BadGateway => (StatusCode::BAD_GATEWAY, "upstream unavailable".to_string()),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
