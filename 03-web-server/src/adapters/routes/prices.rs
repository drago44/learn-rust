use axum::{Json, extract::Path};
use serde::Serialize;

#[derive(Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price_usd: f64,
}

pub async fn get_price(Path(symbol): Path<String>) -> Json<PriceResponse> {
    Json(PriceResponse {
        symbol: symbol.to_uppercase(),
        price_usd: 0.0,
    })
}
