use anyhow::Result;
use serde::Deserialize;

use crate::types::{Balance, Tx};

const API: &str = "https://api.trongrid.io/v1";

// TronGrid повертає дані у { "data": [...], "success": true }
#[derive(Deserialize)]
struct TronResponse<T> {
    data: Vec<T>,
}

// GET /v1/accounts/{address}
#[derive(Deserialize)]
struct TronAccount {
    balance: Option<u64>, // в SUN; відсутній якщо акаунт ще не активований
}

// GET /v1/accounts/{address}/transactions
#[derive(Deserialize)]
struct TronTx {
    #[serde(rename = "txID")]
    tx_id: String,
    raw_data: TronRawData,
    ret: Vec<TronRet>,
}

#[derive(Deserialize)]
struct TronRawData {
    contract: Vec<TronContract>,
    timestamp: u64, // мілісекунди
}

#[derive(Deserialize)]
struct TronContract {
    #[serde(rename = "type")]
    kind: String, // "TransferContract", "TriggerSmartContract", тощо
    parameter: TronParameter,
}

#[derive(Deserialize)]
struct TronParameter {
    value: TronValue,
}

#[derive(Deserialize)]
struct TronValue {
    amount: Option<u64>, // SUN; є тільки у TransferContract
    owner_address: Option<String>,
}

#[derive(Deserialize)]
struct TronRet {
    #[serde(rename = "contractRet")]
    contract_ret: Option<String>, // "SUCCESS" або помилка
}

pub async fn get_balance(address: &str) -> Result<Balance> {
    let url = format!("{API}/accounts/{address}");
    let resp: TronResponse<TronAccount> = reqwest::get(&url).await?.json().await?;

    // Новий акаунт без транзакцій може мати порожній data
    let sun = resp.data.first().and_then(|a| a.balance).unwrap_or(0);

    Ok(Balance {
        address: address.to_string(),
        amount: sun,
        symbol: "TRX".to_string(),
        decimals: 6, // 1 TRX = 10^6 SUN
    })
}

pub async fn get_txs(address: &str) -> Result<Vec<Tx>> {
    let url = format!("{API}/accounts/{address}/transactions?limit=10");
    let resp: TronResponse<TronTx> = reqwest::get(&url).await?.json().await?;

    let txs = resp
        .data
        .into_iter()
        .filter_map(|tx| {
            let contract = tx.raw_data.contract.into_iter().next()?;

            // Обробляємо тільки прості TRX-перекази; смарт-контракти пропускаємо
            if contract.kind != "TransferContract" {
                return None;
            }

            let value = contract.parameter.value;
            let sun = value.amount.unwrap_or(0) as i64;
            let success = tx.ret.first().and_then(|r| r.contract_ret.as_deref()) == Some("SUCCESS");

            // Якщо owner_address — наша адреса, то ми відправник → мінус
            let is_sender = value
                .owner_address
                .as_deref()
                .map(|a| a.eq_ignore_ascii_case(address))
                .unwrap_or(false);

            let amount = if is_sender { -sun } else { sun };

            Some(Tx {
                txid: tx.tx_id,
                amount,
                confirmations: if success { 1 } else { 0 },
                timestamp: Some(tx.raw_data.timestamp / 1000), // мс → секунди
            })
        })
        .collect();

    Ok(txs)
}
