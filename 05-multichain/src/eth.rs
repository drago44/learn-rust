use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use serde::Deserialize;

use crate::types::{Balance, Tx};

const RPC_URL: &str = "https://ethereum.publicnode.com";
const ETHPLORER_API: &str = "https://api.ethplorer.io";

// Ethplorer повертає ETH-транзакції у float-значеннях (вже в ETH, не у Wei)
#[derive(Deserialize)]
struct EthplorerTx {
    timestamp: u64,
    from: String,
    hash: String,
    value: f64,
    success: bool,
}

pub async fn get_balance(address: &str) -> Result<Balance> {
    // Підключаємось до публічної Ethereum ноди через HTTP JSON-RPC
    let provider = ProviderBuilder::new().connect_http(RPC_URL.parse()?);
    let addr: Address = address.parse()?;

    // eth_getBalance повертає Wei у форматі U256 (256-бітне беззнакове ціле)
    let balance_wei: U256 = provider.get_balance(addr).await?;

    // Wei → Gwei: ділимо на 10^9, щоб вмістити в u64
    // 1 ETH = 10^18 Wei = 10^9 Gwei
    let gwei = balance_wei / U256::from(1_000_000_000u64);

    Ok(Balance {
        address: address.to_string(),
        amount: gwei.to::<u64>(),
        symbol: "ETH".to_string(),
        decimals: 9,
    })
}

pub async fn get_txs(address: &str) -> Result<Vec<Tx>> {
    let url = format!("{ETHPLORER_API}/getAddressTransactions/{address}?apiKey=freekey&limit=10");

    let raw: Vec<EthplorerTx> = reqwest::get(&url).await?.json().await?;

    let addr_lower = address.to_lowercase();

    let txs = raw
        .into_iter()
        .filter(|tx| tx.success)
        .map(|tx| {
            // value — в ETH (float), конвертуємо в Gwei для узгодженості з Balance
            let gwei = (tx.value * 1_000_000_000.0) as i64;

            // Якщо ми відправники — мінус, якщо одержувачі — плюс
            let amount = if tx.from.to_lowercase() == addr_lower {
                -gwei
            } else {
                gwei
            };

            Tx {
                txid: tx.hash,
                amount,
                confirmations: 1, // Ethplorer повертає лише підтверджені
                timestamp: Some(tx.timestamp),
            }
        })
        .collect();

    Ok(txs)
}
