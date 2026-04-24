use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub address: String,
    pub amount: u64, // satoshi / gwei / lamport
    pub symbol: String,
    pub decimals: u8, // скільки знаків: BTC=8, ETH=9, SOL=9
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tx {
    pub txid: String,
    pub amount: i64, // + отримано, - відправлено
    pub confirmations: u32,
    pub timestamp: Option<u64>, // Unix секунди; None = unconfirmed
}
