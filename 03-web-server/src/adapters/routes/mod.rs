use axum::{Router, routing::get};

mod health;

pub fn routes() -> Router {
    let v1 = Router::new().route("/health", get(health::health_handler));

    Router::new().nest("/api/v1", v1)
}
