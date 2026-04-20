use crate::adapters::clients::coingecko;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price_usd: f64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn get_price(Path(symbol): Path<String>) -> impl IntoResponse {
    match coingecko::fetch_price(&symbol).await {
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: "CoinGecko unavailable".to_string(),
            }),
        )
            .into_response(),
        Ok(0.0) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Coin '{}' not found", symbol),
            }),
        )
            .into_response(),
        Ok(price) => Json(PriceResponse {
            symbol: symbol.to_uppercase(),
            price_usd: price,
        })
        .into_response(),
    }
}
