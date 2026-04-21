use crate::{dto::coins::CoinInfo, error::AppError, services::coingecko};
use axum::Json;

pub async fn get_coins() -> Result<Json<Vec<CoinInfo>>, AppError> {
    let coins = coingecko::get_coins().await?;
    Ok(Json(coins))
}
