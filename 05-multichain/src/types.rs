use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub address: String,
    pub amount: u64, // satoshi / wei / lamport
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tx {
    pub txid: String,
    pub amount: i64, // + отримано, - відправлено
    pub confirmations: u32,
    pub timestamp: Option<u64>, // Unix секунди; None = unconfirmed
}
