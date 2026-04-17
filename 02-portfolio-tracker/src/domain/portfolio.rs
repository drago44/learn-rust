use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Asset {
    pub symbol: String,
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Portfolio {
    pub assets: HashMap<String, Asset>,
}

impl Portfolio {
    pub fn add(&mut self, symbol: String, amount: f64) {
        let symbol = symbol.to_uppercase();
        self.assets
            .entry(symbol.clone())
            .and_modify(|a| a.amount += amount)
            .or_insert(Asset { symbol, amount });
    }

    pub fn remove(&mut self, symbol: &str) -> bool {
        self.assets.remove(&symbol.to_uppercase()).is_some()
    }

    pub fn assets(&self) -> Vec<&Asset> {
        let mut list: Vec<&Asset> = self.assets.values().collect();
        list.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        list
    }
}
