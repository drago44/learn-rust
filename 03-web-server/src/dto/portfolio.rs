use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AddAssetRequest {
    pub symbol: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct AssetResponse {
    pub id: String,
    pub symbol: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct PortfolioResponse {
    pub id: String,
    pub name: String,
    pub assets: Vec<AssetResponse>,
}
