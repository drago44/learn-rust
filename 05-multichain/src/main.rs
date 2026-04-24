use anyhow::Result;
use clap::{Parser, Subcommand};
use multichain::types::{Balance, Tx};
use multichain::{btc, eth};

#[derive(Parser)]
#[command(name = "chain", about = "Multichain CLI")]
struct Cli {
    #[command(subcommand)]
    chain: Chain,
}

#[derive(Subcommand)]
enum Chain {
    /// Bitcoin — UTXO модель, Mempool.space API
    Btc {
        #[command(subcommand)]
        cmd: ChainCmd,
    },
    /// Ethereum — account модель, alloy + Ethplorer API
    Eth {
        #[command(subcommand)]
        cmd: ChainCmd,
    },
}

// Однакові субкоманди для всіх ланцюгів — balance і txs
#[derive(Subcommand)]
enum ChainCmd {
    /// Показати баланс адреси
    Balance { address: String },
    /// Показати останні транзакції адреси
    Txs { address: String },
}

// AsyncFn — дозволяє передавати async-функції як аргументи
async fn run_cmd(
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.chain {
        // cargo run -- btc balance 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        // cargo run -- btc txs    1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        Chain::Btc { cmd } => run_cmd(cmd, "BTC", 8, btc::get_balance, btc::get_txs).await,

        // cargo run -- eth balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        // cargo run -- eth txs    0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        Chain::Eth { cmd } => run_cmd(cmd, "ETH", 9, eth::get_balance, eth::get_txs).await,
    }
}
