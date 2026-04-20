use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CoinInfo {
    pub id: String,
    pub symbol: String,
    pub name: String,
}
