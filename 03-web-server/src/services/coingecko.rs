use crate::{dto::coins::CoinInfo, error::AppError};
use anyhow::Result;
use std::collections::HashMap;

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("User-Agent", "web-server/0.1".parse().unwrap());
            headers
        })
        .build()
        .unwrap()
}

pub async fn get_coins() -> Result<Vec<CoinInfo>, AppError> {
    build_client()
        .get("https://api.coingecko.com/api/v3/coins/list")
        .send()
        .await
        .map_err(|_| AppError::BadGateway)?
        .json::<Vec<CoinInfo>>()
        .await
        .map_err(|_| AppError::BadGateway)
}

pub async fn get_price(symbol: &str) -> Result<f64, AppError> {
    let id = symbol.to_lowercase();
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        id
    );

    let response = build_client()
        .get(&url)
        .send()
        .await
        .map_err(|_| AppError::BadGateway)?
        .json::<HashMap<String, HashMap<String, f64>>>()
        .await
        .map_err(|_| AppError::BadGateway)?;

    response
        .get(&id)
        .and_then(|r| r.get("usd"))
        .copied()
        .ok_or_else(|| AppError::NotFound(format!("Coin '{}' not found", symbol)))
}
