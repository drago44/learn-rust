use crate::{error::AppError, models::coin::CoinInfo, services::coingecko};
use axum::Json;

pub async fn get_coins() -> Result<Json<Vec<CoinInfo>>, AppError> {
    let coins = coingecko::fetch_coins_list()
        .await
        .map_err(|_| AppError::BadGateway)?;

    Ok(Json(coins))
}
