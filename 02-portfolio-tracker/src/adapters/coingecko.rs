use crate::ports::price_feed::PriceFeedPort;
use anyhow::Result;
use std::collections::HashMap;

pub struct CoinGeckoAdapter;

fn symbol_to_id(symbol: &str) -> String {
    let lower = symbol.to_lowercase();
    match lower.as_str() {
        "btc" => "bitcoin".to_string(),
        "eth" => "ethereum".to_string(),
        "sol" => "solana".to_string(),
        "usdt" => "tether".to_string(),
        "usdc" => "usd-coin".to_string(),
        "bnb" => "binancecoin".to_string(),
        other => other.to_string(),
    }
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("User-Agent", "portfolio-tracker/0.1".parse().unwrap());
            headers
        })
        .build()
        .unwrap()
}

impl PriceFeedPort for CoinGeckoAdapter {
    async fn get_prices(&self, symbols: &[String], currency: &str) -> Result<HashMap<String, f64>> {
        let ids = symbols
            .iter()
            .map(|s| symbol_to_id(s))
            .collect::<Vec<_>>()
            .join(",");

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
            ids, currency
        );

        let response = build_client()
            .get(&url)
            .send()
            .await?
            .json::<HashMap<String, HashMap<String, f64>>>()
            .await?;

        let prices = symbols
            .iter()
            .filter_map(|s| {
                let price = response.get(&symbol_to_id(s))?.get(currency)?;
                Some((s.to_uppercase(), *price))
            })
            .collect();

        Ok(prices)
    }

    // Окремий запит для 24h зміни — буде виконуватись паралельно з get_prices
    async fn get_24h_change(&self, symbols: &[String]) -> Result<HashMap<String, f64>> {
        let ids = symbols
            .iter()
            .map(|s| symbol_to_id(s))
            .collect::<Vec<_>>()
            .join(",");

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
            ids
        );

        let response = build_client()
            .get(&url)
            .send()
            .await?
            .json::<HashMap<String, HashMap<String, f64>>>()
            .await?;

        let changes = symbols
            .iter()
            .filter_map(|s| {
                let change = response.get(&symbol_to_id(s))?.get("usd_24h_change")?;
                Some((s.to_uppercase(), *change))
            })
            .collect();

        Ok(changes)
    }
}
