use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "chain", about = "Multichain CLI")]
pub struct Cli {
    pub chain: Chain,
    #[command(subcommand)]
    pub cmd: ChainCmd,
}

#[derive(ValueEnum, Clone)]
pub enum Chain {
    /// Bitcoin — UTXO модель, Mempool.space API
    Btc,
    /// Ethereum — account модель, alloy + Ethplorer API
    Eth,
    /// Solana — account модель, JSON-RPC
    Sol,
    /// Tron — account модель, TronGrid REST API
    Trx,
}

// Всі субкоманди — parses clap
#[derive(Subcommand)]
pub enum ChainCmd {
    /// Показати баланс адреси
    Balance { address: String },
    /// Показати останні транзакції адреси
    Txs { address: String },
    /// Слідкувати за адресою в реальному часі (WebSocket)
    Watch { address: String },
    /// Відправити токени (ключ з ENV: ETH_PRIVATE_KEY / SOL_PRIVATE_KEY / TRX_PRIVATE_KEY)
    Send { to: String, amount: f64 },
    /// Згенерувати новий тестовий гаманець
    Keygen,
}

// Тільки запити на читання — передається в display::run_cmd
// Не містить Watch / Send / Keygen, тому display.rs не потребує unreachable!
pub enum QueryCmd {
    Balance { address: String },
    Txs { address: String },
}

impl TryFrom<ChainCmd> for QueryCmd {
    type Error = ChainCmd; // повертаємо оригінальну команду якщо не query

    fn try_from(cmd: ChainCmd) -> Result<Self, Self::Error> {
        match cmd {
            ChainCmd::Balance { address } => Ok(QueryCmd::Balance { address }),
            ChainCmd::Txs { address } => Ok(QueryCmd::Txs { address }),
            other => Err(other),
        }
    }
}
