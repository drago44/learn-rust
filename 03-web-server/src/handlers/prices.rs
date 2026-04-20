use crate::{error::AppError, services::coingecko};
use axum::{Json, extract::Path};
use serde::Serialize;

#[derive(Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price_usd: f64,
}

pub async fn get_price(Path(symbol): Path<String>) -> Result<Json<PriceResponse>, AppError> {
    let price = coingecko::fetch_price(&symbol)
        .await
        .map_err(|_| AppError::BadGateway)?;

    if price == 0.0 {
        return Err(AppError::NotFound(format!("Coin '{}' not found", symbol)));
    }

    Ok(Json(PriceResponse {
        symbol: symbol.to_uppercase(),
        price_usd: price,
    }))
}
