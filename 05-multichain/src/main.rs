use multichain::btc;

// #[tokio::main] перетворює async fn main на звичайну — tokio запускає runtime
#[tokio::main]
async fn main() {
    // Відомий гаманець Satoshi Nakamoto (genesis block)
    let address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";

    match btc::get_balance(address).await {
        Ok(balance) => {
            let btc = balance.amount as f64 / 100_000_000.0;
            println!("Address: {}", balance.address);
            println!("Balance: {:.8} {}", btc, balance.symbol);
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    println!("\nTransactions:");
    match btc::get_txs(address).await {
        Ok(txs) => {
            for tx in txs.iter().take(5) {
                let btc = tx.amount as f64 / 100_000_000.0;
                println!(
                    "  {} | {:+.8} BTC | conf: {}",
                    &tx.txid[..8],
                    btc,
                    tx.confirmations
                );
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
