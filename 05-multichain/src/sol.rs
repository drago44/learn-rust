use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_json::json;

use crate::types::{Balance, Tx};

const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

// Solana використовує JSON-RPC 2.0 — всі запити POST на один ендпоінт.
// Відповідь завжди: { "jsonrpc": "2.0", "result": <дані>, "id": 1 }
#[derive(Deserialize)]
struct RpcResponse<T> {
    result: T,
}

// getBalance -> { "context": {...}, "value": 123456789 }
#[derive(Deserialize)]
struct BalanceResult {
    value: u64,
}

// getSignaturesForAddress -> масив об'єктів з підписом та метаданими
#[derive(Deserialize)]
struct SigInfo {
    signature: String,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
    err: Option<serde_json::Value>, // None = успішна, Some = помилка
}

// getTransaction -> об'єкт з meta (баланси до/після) і transaction (акаунти)
#[derive(Deserialize)]
struct TxResult {
    meta: Option<TxMeta>,
    transaction: TxData,
}

#[derive(Deserialize)]
struct TxMeta {
    #[serde(rename = "preBalances")]
    pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    post_balances: Vec<u64>,
}

#[derive(Deserialize)]
struct TxData {
    message: TxMessage,
}

#[derive(Deserialize)]
struct TxMessage {
    #[serde(rename = "accountKeys")]
    account_keys: Vec<String>,
}

pub async fn get_balance(address: &str) -> Result<Balance> {
    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address]
    });

    let resp: RpcResponse<BalanceResult> = client
        .post(RPC_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    Ok(Balance {
        address: address.to_string(),
        amount: resp.result.value, // вже в lamports, без конвертації
        symbol: "SOL".to_string(),
        decimals: 9, // 1 SOL = 10^9 lamports
    })
}

pub async fn get_txs(address: &str) -> Result<Vec<Tx>> {
    let client = reqwest::Client::new();

    // Крок 1: отримуємо список підписів транзакцій для адреси
    let sigs_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignaturesForAddress",
        "params": [address, { "limit": 10 }]
    });

    let sigs_resp: RpcResponse<Vec<SigInfo>> = client
        .post(RPC_URL)
        .json(&sigs_body)
        .send()
        .await?
        .json()
        .await?;

    let mut txs = Vec::new();

    // Крок 2: для кожного підпису отримуємо повну транзакцію
    // щоб порахувати дельту балансу (postBalances - preBalances)
    for sig in sigs_resp.result {
        let tx_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                sig.signature,
                // maxSupportedTransactionVersion потрібен для versioned txs (mainnet)
                { "encoding": "json", "maxSupportedTransactionVersion": 0 }
            ]
        });

        let tx_resp: RpcResponse<Option<TxResult>> = client
            .post(RPC_URL)
            .json(&tx_body)
            .send()
            .await?
            .json()
            .await?;

        let Some(tx) = tx_resp.result else { continue };
        let Some(meta) = tx.meta else { continue };

        // Знаходимо індекс нашої адреси серед учасників транзакції
        let idx = tx
            .transaction
            .message
            .account_keys
            .iter()
            .position(|k| k == address)
            .ok_or_else(|| anyhow!("address not found in tx accounts"))?;

        // Дельта: скільки lamports прийшло (+) або пішло (-) з нашого акаунту
        let delta = meta.post_balances[idx] as i64 - meta.pre_balances[idx] as i64;

        txs.push(Tx {
            txid: sig.signature,
            amount: delta,
            confirmations: if sig.err.is_none() { 1 } else { 0 },
            timestamp: sig.block_time.map(|t| t as u64),
        });
    }

    Ok(txs)
}
