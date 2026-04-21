use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CoinInfo {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price_usd: f64,
}
