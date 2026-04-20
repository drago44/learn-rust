use crate::adapters::clients::coingecko::{self, CoinInfo};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn get_coins() -> impl IntoResponse {
    match coingecko::fetch_coins_list().await {
        Err(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: "CoinGecko unavailable".to_string(),
            }),
        )
            .into_response(),
        Ok(coins) => Json::<Vec<CoinInfo>>(coins).into_response(),
    }
}
