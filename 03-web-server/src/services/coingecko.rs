use crate::models::coin::CoinInfo;
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

pub async fn fetch_coins_list() -> Result<Vec<CoinInfo>> {
    let url = "https://api.coingecko.com/api/v3/coins/list";

    let response = build_client()
        .get(url)
        .send()
        .await?
        .json::<Vec<CoinInfo>>()
        .await?;

    Ok(response)
}

pub async fn fetch_price(symbol: &str) -> Result<f64> {
    let id = symbol.to_lowercase();
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        id
    );

    let response = build_client()
        .get(&url)
        .send()
        .await?
        .json::<HashMap<String, HashMap<String, f64>>>()
        .await?;

    let price = response
        .get(&id)
        .and_then(|r| r.get("usd"))
        .copied()
        .unwrap_or(0.0);

    Ok(price)
}
