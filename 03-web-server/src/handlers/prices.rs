use crate::{dto::coins::PriceResponse, error::AppError, services::coingecko};
use axum::{Json, extract::Path};

pub async fn get_price(Path(symbol): Path<String>) -> Result<Json<PriceResponse>, AppError> {
    let price_usd = coingecko::get_price(&symbol).await?;
    Ok(Json(PriceResponse {
        symbol: symbol.to_uppercase(),
        price_usd,
    }))
}
