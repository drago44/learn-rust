use crate::domain::health::HealthResponse;
use axum::Json;

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
