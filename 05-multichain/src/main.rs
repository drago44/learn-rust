use clap::{Parser, Subcommand};
use multichain::btc;

#[derive(Parser)]
#[command(name = "chain", about = "Multichain CLI")]
struct Cli {
    #[command(subcommand)]
    chain: Chain,
}

// Верхній рівень — який блокчейн. Пізніше додамо Eth і Sol.
#[derive(Subcommand)]
enum Chain {
    Btc {
        #[command(subcommand)]
        cmd: BtcCmd,
    },
}

#[derive(Subcommand)]
enum BtcCmd {
    Balance { address: String },
    Txs { address: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.chain {
        Chain::Btc { cmd } => match cmd {
            BtcCmd::Balance { address } => match btc::get_balance(&address).await {
                Ok(b) => {
                    let amount = b.amount as f64 / 100_000_000.0;
                    println!("{}: {:.8} {}", b.address, amount, b.symbol);
                }
                Err(e) => eprintln!("Error: {e}"),
            },
            BtcCmd::Txs { address } => match btc::get_txs(&address).await {
                Ok(txs) => {
                    for tx in &txs {
                        let amount = tx.amount as f64 / 100_000_000.0;
                        println!(
                            "{} | {:+.8} BTC | conf: {}",
                            &tx.txid[..8],
                            amount,
                            tx.confirmations
                        );
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            },
        },
    }
}
