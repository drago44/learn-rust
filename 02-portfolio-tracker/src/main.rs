mod adapters;
mod domain;
mod ports;

use adapters::cli::{Cli, Command};
use adapters::coingecko::CoinGeckoAdapter;
use adapters::json_storage::JsonStorage;
use anyhow::Result;
use clap::Parser;
use ports::price_feed::PriceFeedPort;
use ports::storage::StoragePort;

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::path::PathBuf::from("temp/portfolio.json");
    let storage = JsonStorage::new(path);
    let mut portfolio = storage.load()?;

    match Cli::parse().command {
        Command::Add { symbol, amount } => {
            portfolio.add(symbol.clone(), amount);
            storage.save(&portfolio)?;
            println!("Додано: {} {}", symbol, amount);
        }
        Command::Remove { symbol } => {
            if portfolio.remove(&symbol) {
                storage.save(&portfolio)?;
                println!("Видалено: {}", symbol);
            } else {
                println!("Не знайдено: {}", symbol);
            }
        }
        Command::List => {
            for asset in portfolio.assets() {
                println!("{}: {}", asset.symbol, asset.amount);
            }
        }
        Command::Total => {
            let symbols: Vec<String> = portfolio
                .assets()
                .iter()
                .map(|a| a.symbol.clone())
                .collect();

            if symbols.is_empty() {
                println!("Портфель порожній");
                return Ok(());
            }

            // Список доступних валют
            let currencies = [
                ("1. USD — долар США", "usd", "$"),
                ("2. EUR — євро", "eur", "€"),
                ("3. UAH — гривня", "uah", "₴"),
                ("4. GBP — фунт стерлінгів", "gbp", "£"),
                ("5. BTC — біткоін", "btc", "₿"),
            ];

            let options: Vec<&str> = currencies.iter().map(|(label, _, _)| *label).collect();

            let selection = dialoguer::Select::new()
                .with_prompt("Оберіть валюту")
                .items(&options)
                .default(0)
                .interact()?;

            let (_, cur, cur_sign) = currencies[selection];

            let feed = CoinGeckoAdapter;

            // Два запити виконуються паралельно — не чекаємо першого щоб почати другий
            let (prices_result, changes_result) = tokio::join!(
                feed.get_prices(&symbols, cur),
                feed.get_24h_change(&symbols)
            );

            let prices = prices_result?;
            let changes = changes_result?;

            let mut total = 0.0;
            for asset in portfolio.assets() {
                if let Some(price) = prices.get(&asset.symbol) {
                    let value = asset.amount * price;
                    let change = changes.get(&asset.symbol).copied().unwrap_or(0.0);
                    let arrow = if change >= 0.0 { "▲" } else { "▼" };
                    println!(
                        "{}: {} × {}{:.2} = {}{:.2}  {} {:.2}%",
                        asset.symbol, asset.amount, cur_sign, price, cur_sign, value, arrow, change
                    );
                    total += value;
                }
            }
            println!("─────────────────");
            println!("Разом: {}{:.2}", cur_sign, total);
        }
    }

    Ok(())
}
