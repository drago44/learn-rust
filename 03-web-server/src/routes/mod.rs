use crate::{handlers, state::AppState};
use axum::{Router, routing::get};
use sqlx::SqlitePool;

pub fn routes(pool: SqlitePool) -> Router {
    let v1 = Router::new()
        .route("/coins", get(handlers::coins::get_coins))
        .route("/prices/{symbol}", get(handlers::prices::get_price))
        .route("/health", get(handlers::health::health_handler));

    Router::new()
        .nest("/api/v1", v1)
        .with_state(AppState { pool })
}
