use axum::{Router, routing::get};
use sqlx::SqlitePool;

mod coins;
mod health;
mod prices;

pub fn routes(pool: SqlitePool) -> Router {
    let v1 = Router::new()
        .route("/coins", get(coins::get_coins))
        .route("/prices/{symbol}", get(prices::get_price))
        .route("/health", get(health::health_handler));

    Router::new().nest("/api/v1", v1).with_state(pool)
}
