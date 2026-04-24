mod cli;
mod display;

use clap::Parser;
use cli::{Chain, ChainCmd, Cli};
use multichain::{btc, eth, sol, trx};

#[tokio::main]
async fn main() {
    let Cli { chain, cmd } = Cli::parse();

    // watch обробляємо окремо — це нескінченний цикл, не вписується в run_cmd
    if let ChainCmd::Watch { ref address } = cmd {
        let result = match chain {
            // cargo run -- btc watch 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
            Chain::Btc => btc::watch(address).await,
            // cargo run -- eth watch 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
            Chain::Eth => eth::watch(address).await,
            // cargo run -- sol watch 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
            Chain::Sol => sol::watch(address).await,
            Chain::Trx => {
                eprintln!("TRX watch не підтримується — TronGrid не має публічного WebSocket");
                return;
            }
        };
        if let Err(e) = result {
            eprintln!("Error: {e}");
        }
        return;
    }

    match chain {
        // cargo run -- btc balance 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        // cargo run -- btc txs    1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        Chain::Btc => display::run_cmd(cmd, "BTC", 8, btc::get_balance, btc::get_txs).await,

        // cargo run -- eth balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        // cargo run -- eth txs    0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        Chain::Eth => display::run_cmd(cmd, "ETH", 9, eth::get_balance, eth::get_txs).await,

        // cargo run -- sol balance 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
        // cargo run -- sol txs    9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
        Chain::Sol => display::run_cmd(cmd, "SOL", 9, sol::get_balance, sol::get_txs).await,

        // cargo run -- trx balance TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW
        // cargo run -- trx txs    TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW
        Chain::Trx => display::run_cmd(cmd, "TRX", 6, trx::get_balance, trx::get_txs).await,
    }
}
