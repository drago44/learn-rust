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

#[derive(Subcommand)]
pub enum ChainCmd {
    /// Показати баланс адреси
    Balance { address: String },
    /// Показати останні транзакції адреси
    Txs { address: String },
}
