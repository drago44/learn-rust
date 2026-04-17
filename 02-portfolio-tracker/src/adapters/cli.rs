use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "portfolio", about = "Crypto portfolio Tracker")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Add { symbol: String, amount: f64 },
    Remove { symbol: String },
    List,
    Total,
}
