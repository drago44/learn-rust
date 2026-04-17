use anyhow::Result;
use std::collections::HashMap;

pub trait PriceFeedPort {
    async fn get_prices(&self, symbols: &[String], currency: &str) -> Result<HashMap<String, f64>>;
    async fn get_24h_change(&self, symbols: &[String]) -> Result<HashMap<String, f64>>;
}
