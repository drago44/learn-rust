use crate::dto::health::HealthResponse;
use axum::Json;

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
