use anyhow::Result;
use serde::Deserialize;

use crate::types::Balance;

const API: &str = "https://mempool.space/api";

// Внутрішні типи — точно відповідають JSON від Mempool.space.
// Приватні, бо назовні віддаємо лише Balance і Tx.

#[derive(Deserialize)]
struct MempoolAddress {
    chain_stats: MempoolStats,   // підтверджені транзакції
    mempool_stats: MempoolStats, // ще в очікуванні
}

#[derive(Deserialize)]
struct MempoolStats {
    funded_txo_sum: u64, // сума всіх вхідних satoshi
    spent_txo_sum: u64,  // сума всіх витрачених satoshi
}

pub async fn get_balance(address: &str) -> Result<Balance> {
    let url = format!("{API}/address/{address}");

    // .await чекає відповідь, ? повертає помилку вгору якщо є
    let resp: MempoolAddress = reqwest::get(&url).await?.json().await?;

    let confirmed = resp
        .chain_stats
        .funded_txo_sum
        .saturating_sub(resp.chain_stats.spent_txo_sum);

    let unconfirmed = resp
        .mempool_stats
        .funded_txo_sum
        .saturating_sub(resp.mempool_stats.spent_txo_sum);

    Ok(Balance {
        address: address.to_string(),
        amount: confirmed + unconfirmed,
        symbol: "BTC".to_string(),
    })
}
