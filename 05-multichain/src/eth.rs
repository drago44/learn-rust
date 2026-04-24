use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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

pub async fn watch(address: &str) -> Result<()> {
    // PublicNode підтримує WebSocket з eth_subscribe
    let (mut ws, _) = connect_async("wss://ethereum.publicnode.com").await?;

    // newHeads — отримуємо кожен новий блок (ETH не має адресних підписок без платного RPC)
    ws.send(Message::text(
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        })
        .to_string(),
    ))
    .await?;

    println!("Watching ETH новi блоки (адреса: {address})\n");

    while let Some(msg) = ws.next().await {
        let text = match msg? {
            Message::Text(t) => t,
            _ => continue,
        };

        let v: serde_json::Value = serde_json::from_str(&text)?;

        // Після підписки приходить підтвердження, потім — notification по кожному блоку
        if let Some(block) = v.get("params").and_then(|p| p.get("result")) {
            let number = block.get("number").and_then(|n| n.as_str()).unwrap_or("?");
            let hash = block.get("hash").and_then(|h| h.as_str()).unwrap_or("?");
            println!("Новий блок {} | {}", number, &hash[..10]);
        }
    }

    Ok(())
}
