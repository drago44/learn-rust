mod cli;
mod display;

use clap::Parser;
use cli::{Chain, ChainCmd, Cli, QueryCmd};
use multichain::{btc, eth, sol, trx};

#[tokio::main]
async fn main() {
    let Cli { chain, cmd } = Cli::parse();

    // Спочатку пробуємо конвертувати в QueryCmd (Balance / Txs)
    // Якщо не вдається — це Watch / Send / Keygen
    let query_cmd = match QueryCmd::try_from(cmd) {
        Ok(q) => q,
        Err(action) => {
            handle_action(chain, action).await;
            return;
        }
    };

    match chain {
        // cargo run -- btc balance 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        // cargo run -- btc txs    1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        Chain::Btc => display::run_cmd(query_cmd, "BTC", 8, btc::get_balance, btc::get_txs).await,

        // cargo run -- eth balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        // cargo run -- eth txs    0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
        Chain::Eth => display::run_cmd(query_cmd, "ETH", 9, eth::get_balance, eth::get_txs).await,

        // cargo run -- sol balance 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
        // cargo run -- sol txs    9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
        Chain::Sol => display::run_cmd(query_cmd, "SOL", 9, sol::get_balance, sol::get_txs).await,

        // cargo run -- trx balance TCrSPhg8ERaeu3mzNesq92TP4fHyjoKWNh && cargo run -- trx balance TB1sPKUDDpc9UAP9TF9Fo3puJCGSRQD9z8
        // cargo run -- trx txs    TCrSPhg8ERaeu3mzNesq92TP4fHyjoKWNh
        Chain::Trx => display::run_cmd(query_cmd, "TRX", 6, trx::get_balance, trx::get_txs).await,
    }
}

async fn handle_action(chain: Chain, cmd: ChainCmd) {
    match cmd {
        ChainCmd::Watch { address } => {
            let result = match chain {
                // cargo run -- btc watch 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
                Chain::Btc => btc::watch(&address).await,
                // cargo run -- eth watch 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
                Chain::Eth => eth::watch(&address).await,
                // cargo run -- sol watch 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
                Chain::Sol => sol::watch(&address).await,
                Chain::Trx => {
                    eprintln!("TRX watch не підтримується");
                    return;
                }
            };
            if let Err(e) = result {
                eprintln!("Error: {e}");
            }
        }

        ChainCmd::Send { to, amount } => {
            let result = match chain {
                // ETH_PRIVATE_KEY=0x... cargo run -- eth send <to> <amount>
                Chain::Eth => {
                    let key = env_key("ETH_PRIVATE_KEY");
                    eth::send(&to, amount, &key).await
                }
                // SOL_PRIVATE_KEY=<base58> cargo run -- sol send <to> <amount>
                Chain::Sol => {
                    let key = env_key("SOL_PRIVATE_KEY");
                    sol::send(&to, amount, &key).await
                }
                // TRX_PRIVATE_KEY=<hex> cargo run -- trx send <to> <amount>
                Chain::Trx => {
                    let key = env_key("TRX_PRIVATE_KEY");
                    trx::send(&to, amount, &key).await
                }
                Chain::Btc => {
                    eprintln!("BTC send не підтримується");
                    return;
                }
            };
            match result {
                Ok(hash) => println!("Tx: {hash}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }

        ChainCmd::Keygen => {
            let (key, address) = match chain {
                // cargo run -- eth keygen
                Chain::Eth => eth::keygen(),
                // cargo run -- sol keygen
                Chain::Sol => sol::keygen(),
                // cargo run -- trx keygen
                Chain::Trx => trx::keygen(),
                Chain::Btc => {
                    eprintln!("BTC keygen не підтримується");
                    return;
                }
            };
            let env_name = match chain {
                Chain::Eth => "ETH_PRIVATE_KEY",
                Chain::Sol => "SOL_PRIVATE_KEY",
                Chain::Trx => "TRX_PRIVATE_KEY",
                Chain::Btc => "",
            };
            println!("Address:     {address}");
            println!("Private key: {key}");
            println!();
            println!("# для відправки:");
            println!("{env_name}={key} cargo run -- ... send <to> <amount>");
            println!();
            println!("УВАГА: тільки для тестових мереж!");
        }

        // Balance і Txs ніколи сюди не потрапляють — оброблені через QueryCmd
        ChainCmd::Balance { .. } | ChainCmd::Txs { .. } => unreachable!(),
    }
}

fn env_key(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        eprintln!("Потрібна ENV: {name}");
        std::process::exit(1);
    })
}
