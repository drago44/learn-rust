use clap::{Parser, Subcommand};
use crypto_primitives::{demo, hdwallet, keys, mnemonic, signing};

#[derive(Parser)]
#[command(name = "wallet", about = "Crypto wallet CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Генерує нову мнемоніку та адреси для ETH, BTC, SOL
    New,
    /// Деривує адреси по індексу з існуючої мнемоніки
    Derive {
        #[arg(short, long, default_value = "0")]
        index: u32,
    },
    /// Підписує повідомлення приватним ключем з мнемоніки
    Sign {
        #[arg(short, long)]
        message: String,
    },
    /// Генерує M-of-N мультисиг адресу (P2SH), починається з '3'
    Multisig {
        #[arg(short, long, default_value = "2")]
        m: usize,
        #[arg(short, long, default_value = "3")]
        n: usize,
    },
    /// Запускає демо всіх криптографічних примітивів
    Demo,
}

//   cargo run -- new
//   cargo run -- derive --index 1
//   cargo run -- sign --message "hello"
//   cargo run -- demo
// Читає рядки з stdin доки не набереться 12 або 24 слова.
// Вирішує проблему коли мнемоніка переноситься на кілька рядків при вставці.
fn read_mnemonic() -> String {
    let mut words: Vec<String> = Vec::new();
    for line in std::io::stdin().lines() {
        let line = line.unwrap();
        words.extend(line.split_whitespace().map(|w| w.to_string()));
        if words.len() >= 12 {
            break;
        }
    }
    words.join(" ")
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::New => {
            let m = mnemonic::generate_mnemonic();
            println!("Mnemonic:    {}", m);
            let seed = m.to_seed("");

            let (_, eth_vk) = hdwallet::derive_eth_keypair(&seed, 0);
            println!("ETH:         {}", keys::eth_address_checksum(&eth_vk));
            println!("BTC P2PKH:   {}", keys::btc_address_p2pkh(&eth_vk));
            println!("BTC P2WPKH:  {}", keys::btc_address_p2wpkh(&eth_vk));
            println!("BTC P2TR:    {}", keys::btc_address_p2tr(&eth_vk));

            let sol = hdwallet::derive_sol_keypair(&seed, 0);
            println!(
                "SOL:         {}",
                keys::solana_address(&sol.verifying_key())
            );
        }

        Command::Derive { index } => {
            println!("Enter mnemonic:");
            let phrase = read_mnemonic();

            let m = mnemonic::mnemonic_from_phrase(&phrase).expect("Невалідна мнемоніка");
            let seed = m.to_seed("");

            let (_, eth_vk) = hdwallet::derive_eth_keypair(&seed, index);
            println!(
                "ETH[{}]:      {}",
                index,
                keys::eth_address_checksum(&eth_vk)
            );
            println!("BTC P2PKH:   {}", keys::btc_address_p2pkh(&eth_vk));
            println!("BTC P2WPKH:  {}", keys::btc_address_p2wpkh(&eth_vk));
            println!("BTC P2TR:    {}", keys::btc_address_p2tr(&eth_vk));

            let sol = hdwallet::derive_sol_keypair(&seed, index);
            println!(
                "SOL[{}]:      {}",
                index,
                keys::solana_address(&sol.verifying_key())
            );
        }

        Command::Sign { message } => {
            println!("Enter mnemonic:");
            let phrase = read_mnemonic();

            let m = mnemonic::mnemonic_from_phrase(&phrase).expect("Невалідна мнемоніка");
            let seed = m.to_seed("");

            let (signing_key, verifying_key) = hdwallet::derive_eth_keypair(&seed, 0);
            let sig = signing::sign_message(&signing_key, &message);
            println!("Message:     {}", message);
            println!("Signature:   {}", hex::encode(sig.to_bytes()));
            println!(
                "Valid:       {}",
                signing::verify_message(&verifying_key, &message, &sig)
            );
        }

        Command::Multisig { m, n } => {
            println!("Generating {m}-of-{n} multisig...\n");
            let mut pub_keys = vec![];
            for i in 0..n {
                println!("Key [{i}]:");
                let (_, vk) = keys::generate_secp256k1_keypair();
                pub_keys.push(vk);
                println!();
            }
            println!(
                "Multisig {m}-of-{n}: {}",
                keys::btc_address_multisig(&pub_keys, m)
            );
        }

        Command::Demo => demo::run(),
    }
}
