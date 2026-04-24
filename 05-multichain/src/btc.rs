use anyhow::Result;
use serde::Deserialize;

use crate::types::{Balance, Tx};

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

// Структури для /txs ендпоінту

#[derive(Deserialize)]
struct MempoolTx {
    txid: String,
    status: MempoolStatus,
    vin: Vec<MempoolVin>,
    vout: Vec<MempoolVout>,
}

#[derive(Deserialize)]
struct MempoolStatus {
    confirmed: bool,
    #[serde(default)] // може бути відсутнє якщо unconfirmed
    block_time: Option<u64>,
    #[serde(default)]
    block_height: Option<u32>,
}

#[derive(Deserialize)]
struct MempoolVin {
    prevout: Option<MempoolPrevout>, // coinbase транзакції не мають prevout
}

#[derive(Deserialize)]
struct MempoolPrevout {
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Deserialize)]
struct MempoolVout {
    scriptpubkey_address: Option<String>,
    value: u64,
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
        decimals: 8,
    })
}

pub async fn get_txs(address: &str) -> Result<Vec<Tx>> {
    let url = format!("{API}/address/{address}/txs");

    let raw: Vec<MempoolTx> = reqwest::get(&url).await?.json().await?;

    let txs = raw
        .into_iter()
        .map(|tx| {
            // Скільки satoshi надійшло на нашу адресу в цій транзакції
            let received: u64 = tx
                .vout
                .iter()
                .filter(|o| o.scriptpubkey_address.as_deref() == Some(address))
                .map(|o| o.value)
                .sum();

            // Скільки ми витратили (наша адреса була відправником)
            let sent: u64 = tx
                .vin
                .iter()
                .filter_map(|i| i.prevout.as_ref())
                .filter(|p| p.scriptpubkey_address.as_deref() == Some(address))
                .map(|p| p.value)
                .sum();

            Tx {
                txid: tx.txid,
                amount: received as i64 - sent as i64, // + отримано, - відправлено
                confirmations: if tx.status.confirmed {
                    tx.status.block_height.unwrap_or(1)
                } else {
                    0
                },
                timestamp: tx.status.block_time,
            }
        })
        .collect();

    Ok(txs)
}
