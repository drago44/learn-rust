use anyhow::{Result, anyhow};
use ed25519_dalek::{Signer, SigningKey};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::types::{Balance, Tx};

const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

// Solana використовує JSON-RPC 2.0 — всі запити POST на один ендпоінт.
// Відповідь завжди: { "jsonrpc": "2.0", "result": <дані>, "id": 1 }
#[derive(Deserialize)]
struct RpcResponse<T> {
    result: T,
}

// getBalance -> { "context": {...}, "value": 123456789 }
#[derive(Deserialize)]
struct BalanceResult {
    value: u64,
}

// getSignaturesForAddress -> масив об'єктів з підписом та метаданими
#[derive(Deserialize)]
struct SigInfo {
    signature: String,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
    err: Option<serde_json::Value>, // None = успішна, Some = помилка
}

// getTransaction -> об'єкт з meta (баланси до/після) і transaction (акаунти)
#[derive(Deserialize)]
struct TxResult {
    meta: Option<TxMeta>,
    transaction: TxData,
}

#[derive(Deserialize)]
struct TxMeta {
    #[serde(rename = "preBalances")]
    pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    post_balances: Vec<u64>,
}

#[derive(Deserialize)]
struct TxData {
    message: TxMessage,
}

#[derive(Deserialize)]
struct TxMessage {
    #[serde(rename = "accountKeys")]
    account_keys: Vec<String>,
}

pub async fn get_balance(address: &str) -> Result<Balance> {
    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address]
    });

    let resp: RpcResponse<BalanceResult> = client
        .post(RPC_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    Ok(Balance {
        address: address.to_string(),
        amount: resp.result.value, // вже в lamports, без конвертації
        symbol: "SOL".to_string(),
        decimals: 9, // 1 SOL = 10^9 lamports
    })
}

pub async fn get_txs(address: &str) -> Result<Vec<Tx>> {
    let client = reqwest::Client::new();

    // Крок 1: отримуємо список підписів транзакцій для адреси
    let sigs_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignaturesForAddress",
        "params": [address, { "limit": 10 }]
    });

    let sigs_resp: RpcResponse<Vec<SigInfo>> = client
        .post(RPC_URL)
        .json(&sigs_body)
        .send()
        .await?
        .json()
        .await?;

    let mut txs = Vec::new();

    // Крок 2: для кожного підпису отримуємо повну транзакцію
    // щоб порахувати дельту балансу (postBalances - preBalances)
    for sig in sigs_resp.result {
        let tx_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                sig.signature,
                // maxSupportedTransactionVersion потрібен для versioned txs (mainnet)
                { "encoding": "json", "maxSupportedTransactionVersion": 0 }
            ]
        });

        let tx_resp: RpcResponse<Option<TxResult>> = client
            .post(RPC_URL)
            .json(&tx_body)
            .send()
            .await?
            .json()
            .await?;

        let Some(tx) = tx_resp.result else { continue };
        let Some(meta) = tx.meta else { continue };

        // Знаходимо індекс нашої адреси серед учасників транзакції
        let idx = tx
            .transaction
            .message
            .account_keys
            .iter()
            .position(|k| k == address)
            .ok_or_else(|| anyhow!("address not found in tx accounts"))?;

        // Дельта: скільки lamports прийшло (+) або пішло (-) з нашого акаунту
        let delta = meta.post_balances[idx] as i64 - meta.pre_balances[idx] as i64;

        txs.push(Tx {
            txid: sig.signature,
            amount: delta,
            confirmations: if sig.err.is_none() { 1 } else { 0 },
            timestamp: sig.block_time.map(|t| t as u64),
        });
    }

    Ok(txs)
}

pub async fn watch(address: &str) -> Result<()> {
    let (mut ws, _) = connect_async("wss://api.mainnet-beta.solana.com").await?;

    // accountSubscribe — Solana нативно підтримує підписку на конкретний акаунт
    ws.send(Message::text(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "accountSubscribe",
            "params": [address, {"encoding": "base64", "commitment": "confirmed"}]
        })
        .to_string(),
    ))
    .await?;

    println!("Watching SOL {address} — очікуємо зміни балансу...\n");

    while let Some(msg) = ws.next().await {
        let text = match msg? {
            Message::Text(t) => t,
            _ => continue,
        };

        let v: serde_json::Value = serde_json::from_str(&text)?;

        // Notification приходить коли баланс акаунту змінюється
        if let Some(value) = v
            .get("params")
            .and_then(|p| p.get("result"))
            .and_then(|r| r.get("value"))
        {
            let lamports = value.get("lamports").and_then(|l| l.as_u64()).unwrap_or(0);
            let sol = lamports as f64 / 1_000_000_000.0;
            println!("Баланс змінився: {sol:.9} SOL");
        }
    }

    Ok(())
}

// Solana використовує compact-u16 для довжин масивів у серіалізації транзакцій
fn compact_u16(n: u16) -> Vec<u8> {
    match n {
        0..=127 => vec![n as u8],
        128..=16383 => vec![(n as u8 & 0x7f) | 0x80, (n >> 7) as u8],
        _ => vec![
            (n as u8 & 0x7f) | 0x80,
            ((n >> 7) as u8 & 0x7f) | 0x80,
            (n >> 14) as u8,
        ],
    }
}

// Будуємо байти повідомлення транзакції (те що підписується)
fn build_message(payer: &[u8; 32], to: &[u8; 32], lamports: u64, blockhash: &[u8; 32]) -> Vec<u8> {
    // SystemProgram — всі нулі (base58: 11111111111111111111111111111111)
    const SYSTEM_PROGRAM: [u8; 32] = [0u8; 32];

    let mut msg = Vec::new();

    // Header: 1 підписант, 0 readonly-підписантів, 1 readonly без підпису (system program)
    msg.extend_from_slice(&[1u8, 0, 1]);

    // Акаунти: payer (index 0), to (index 1), system program (index 2)
    msg.extend_from_slice(&compact_u16(3));
    msg.extend_from_slice(payer);
    msg.extend_from_slice(to);
    msg.extend_from_slice(&SYSTEM_PROGRAM);

    // Recent blockhash
    msg.extend_from_slice(blockhash);

    // Одна інструкція
    msg.extend_from_slice(&compact_u16(1));

    // Інструкція SystemProgram::Transfer:
    msg.push(2); // program_id_index = system program (index 2)
    msg.extend_from_slice(&compact_u16(2)); // 2 акаунти: [payer, to]
    msg.extend_from_slice(&[0u8, 1]); // індекси акаунтів
    msg.extend_from_slice(&compact_u16(12)); // data: 4 + 8 байт
    msg.extend_from_slice(&2u32.to_le_bytes()); // Transfer = variant 2
    msg.extend_from_slice(&lamports.to_le_bytes());

    msg
}

pub async fn send(to: &str, amount_sol: f64, private_key: &str) -> Result<String> {
    // Solana ключ — base58-рядок із 64 байт (32 seed + 32 pubkey)
    let key_bytes = bs58::decode(private_key).into_vec()?;
    let seed: [u8; 32] = key_bytes[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(&seed);
    let payer_pubkey: [u8; 32] = signing_key.verifying_key().to_bytes();

    let to_bytes = bs58::decode(to).into_vec()?;
    let to_pubkey: [u8; 32] = to_bytes
        .try_into()
        .map_err(|_| anyhow!("invalid to address"))?;

    let lamports = (amount_sol * 1_000_000_000.0) as u64;

    let client = reqwest::Client::new();
    const DEVNET: &str = "https://api.devnet.solana.com";

    // Отримуємо свіжий blockhash — обов'язковий для кожної транзакції
    let bh_resp: serde_json::Value = client
        .post(DEVNET)
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"getLatestBlockhash","params":[]}))
        .send()
        .await?
        .json()
        .await?;

    let blockhash_str = bh_resp["result"]["value"]["blockhash"]
        .as_str()
        .ok_or_else(|| anyhow!("no blockhash"))?;
    let blockhash_bytes = bs58::decode(blockhash_str).into_vec()?;
    let blockhash: [u8; 32] = blockhash_bytes
        .try_into()
        .map_err(|_| anyhow!("bad blockhash"))?;

    // Будуємо і підписуємо транзакцію
    let message = build_message(&payer_pubkey, &to_pubkey, lamports, &blockhash);
    let signature = signing_key.sign(&message);
    let sig_bytes: [u8; 64] = signature.to_bytes();

    // Повна транзакція: compact_u16(1) + signature + message
    let mut tx_bytes = compact_u16(1);
    tx_bytes.extend_from_slice(&sig_bytes);
    tx_bytes.extend_from_slice(&message);

    // Відправляємо base64-encoded транзакцію
    use base64::{Engine, engine::general_purpose::STANDARD};
    let tx_b64 = STANDARD.encode(&tx_bytes);

    let send_resp: serde_json::Value = client
        .post(DEVNET)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [tx_b64, {"encoding": "base64"}]
        }))
        .send()
        .await?
        .json()
        .await?;

    if let Some(err) = send_resp.get("error") {
        return Err(anyhow!("{}", err));
    }

    let sig = send_resp["result"]
        .as_str()
        .ok_or_else(|| anyhow!("no signature in response"))?;
    Ok(sig.to_string())
}

// Повертає (private_key_base58, address_base58)
pub fn keygen() -> (String, String) {
    let seed: [u8; 32] = rand::random();
    let signing_key = SigningKey::from_bytes(&seed);
    let pubkey = signing_key.verifying_key().to_bytes();

    // Solana зберігає ключ як 64 байти: seed(32) + pubkey(32)
    let mut full_key = [0u8; 64];
    full_key[..32].copy_from_slice(&seed);
    full_key[32..].copy_from_slice(&pubkey);

    (
        bs58::encode(full_key).into_string(),
        bs58::encode(pubkey).into_string(),
    )
}
