mod cli;
mod display;

use clap::Parser;
use cli::{Chain, Cli};
use multichain::{btc, eth, sol, trx};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.chain {
        // cargo run -- btc balance 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        // cargo run -- btc txs    1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        Chain::Btc => display::run_cmd(cli.cmd, "BTC", 8, btc::get_balance, btc::get_txs).await,

        // cargo run -- eth balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        // cargo run -- eth txs    0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        Chain::Eth => display::run_cmd(cli.cmd, "ETH", 9, eth::get_balance, eth::get_txs).await,

        // cargo run -- sol balance 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
        // cargo run -- sol txs    9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
        Chain::Sol => display::run_cmd(cli.cmd, "SOL", 9, sol::get_balance, sol::get_txs).await,

        // cargo run -- trx balance TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW
        // cargo run -- trx txs    TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW
        Chain::Trx => display::run_cmd(cli.cmd, "TRX", 6, trx::get_balance, trx::get_txs).await,
    }
}
