use anyhow::Result;
use multichain::types::{Balance, Tx};

use crate::cli::ChainCmd;

pub async fn run_cmd(
    cmd: ChainCmd,
    symbol: &str,
    decimals: u8,
    get_balance: impl AsyncFn(&str) -> Result<Balance>,
    get_txs: impl AsyncFn(&str) -> Result<Vec<Tx>>,
) {
    match cmd {
        ChainCmd::Balance { address } => match get_balance(&address).await {
            Ok(b) => print_balance(&b),
            Err(e) => eprintln!("Error: {e}"),
        },
        ChainCmd::Txs { address } => match get_txs(&address).await {
            Ok(txs) => print_txs(&txs, symbol, decimals),
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

fn print_balance(b: &Balance) {
    let divisor = 10f64.powi(b.decimals as i32);
    let amount = b.amount as f64 / divisor;
    println!(
        "{}: {:.decimals$} {}",
        b.address,
        amount,
        b.symbol,
        decimals = b.decimals as usize
    );
}

fn print_txs(txs: &[Tx], symbol: &str, decimals: u8) {
    let divisor = 10f64.powi(decimals as i32);
    for tx in txs {
        let amount = tx.amount as f64 / divisor;
        let meta = match tx.timestamp {
            Some(ts) => format!("ts: {ts}"),
            None => format!("conf: {}", tx.confirmations),
        };
        println!(
            "{} | {:+.*} {} | {meta}",
            &tx.txid[..8],
            decimals as usize,
            amount,
            symbol
        );
    }
}
