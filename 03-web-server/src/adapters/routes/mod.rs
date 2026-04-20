use axum::{Router, routing::get};

mod coins;
mod health;
mod prices;

pub fn routes() -> Router {
    let v1 = Router::new()
        .route("/coins", get(coins::get_coins))
        .route("/prices/{symbol}", get(prices::get_price))
        .route("/health", get(health::health_handler));

    Router::new().nest("/api/v1", v1)
}
