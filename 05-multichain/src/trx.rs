use anyhow::{Result, anyhow};
use k256::ecdsa::SigningKey;
use serde::Deserialize;

use crate::types::{Balance, Tx};

const API: &str = "https://nile.trongrid.io/v1";

// TronGrid повертає дані у { "data": [...], "success": true }
#[derive(Deserialize)]
struct TronResponse<T> {
    data: Vec<T>,
}

// GET /v1/accounts/{address}
#[derive(Deserialize)]
struct TronAccount {
    balance: Option<u64>, // в SUN; відсутній якщо акаунт ще не активований
}

// GET /v1/accounts/{address}/transactions
#[derive(Deserialize)]
struct TronTx {
    #[serde(rename = "txID")]
    tx_id: String,
    raw_data: TronRawData,
    ret: Vec<TronRet>,
}

#[derive(Deserialize)]
struct TronRawData {
    contract: Vec<TronContract>,
    timestamp: u64, // мілісекунди
}

#[derive(Deserialize)]
struct TronContract {
    #[serde(rename = "type")]
    kind: String, // "TransferContract", "TriggerSmartContract", тощо
    parameter: TronParameter,
}

#[derive(Deserialize)]
struct TronParameter {
    value: TronValue,
}

#[derive(Deserialize)]
struct TronValue {
    amount: Option<u64>, // SUN; є тільки у TransferContract
    owner_address: Option<String>,
}

#[derive(Deserialize)]
struct TronRet {
    #[serde(rename = "contractRet")]
    contract_ret: Option<String>, // "SUCCESS" або помилка
}

pub async fn get_balance(address: &str) -> Result<Balance> {
    let url = format!("{API}/accounts/{address}");
    let resp: TronResponse<TronAccount> = reqwest::get(&url).await?.json().await?;

    // Новий акаунт без транзакцій може мати порожній data
    let sun = resp.data.first().and_then(|a| a.balance).unwrap_or(0);

    Ok(Balance {
        address: address.to_string(),
        amount: sun,
        symbol: "TRX".to_string(),
        decimals: 6, // 1 TRX = 10^6 SUN
    })
}

pub async fn get_txs(address: &str) -> Result<Vec<Tx>> {
    let url = format!("{API}/accounts/{address}/transactions?limit=10");
    let resp: TronResponse<TronTx> = reqwest::get(&url).await?.json().await?;

    let txs = resp
        .data
        .into_iter()
        .filter_map(|tx| {
            let contract = tx.raw_data.contract.into_iter().next()?;

            // Обробляємо тільки прості TRX-перекази; смарт-контракти пропускаємо
            if contract.kind != "TransferContract" {
                return None;
            }

            let value = contract.parameter.value;
            let sun = value.amount.unwrap_or(0) as i64;
            let success = tx.ret.first().and_then(|r| r.contract_ret.as_deref()) == Some("SUCCESS");

            // Якщо owner_address — наша адреса, то ми відправник → мінус
            let is_sender = value
                .owner_address
                .as_deref()
                .map(|a| a.eq_ignore_ascii_case(address))
                .unwrap_or(false);

            let amount = if is_sender { -sun } else { sun };

            Some(Tx {
                txid: tx.tx_id,
                amount,
                confirmations: if success { 1 } else { 0 },
                timestamp: Some(tx.raw_data.timestamp / 1000), // мс → секунди
            })
        })
        .collect();

    Ok(txs)
}

pub async fn send(to: &str, amount_trx: f64, private_key: &str) -> Result<String> {
    // Shasta — Tron testnet
    const NILE: &str = "https://nile.trongrid.io";

    let client = reqwest::Client::new();

    // Крок 1: TronGrid створює unsigned транзакцію і повертає txID (хеш для підпису)
    let create_resp: serde_json::Value = client
        .post(format!("{NILE}/wallet/createtransaction"))
        .json(&serde_json::json!({
            "owner_address": to_hex_address(private_key)?,
            "to_address": to_hex_address_from_base58(to)?,
            "amount": (amount_trx * 1_000_000.0) as u64  // TRX → SUN
        }))
        .send()
        .await?
        .json()
        .await?;

    let tx_id_hex = create_resp["txID"]
        .as_str()
        .ok_or_else(|| anyhow!("no txID: {create_resp}"))?;

    // Крок 2: підписуємо txID (32 байти хешу) ключем secp256k1
    let key_bytes = hex::decode(private_key)?;
    let signing_key = SigningKey::from_slice(&key_bytes)?;
    let hash = hex::decode(tx_id_hex)?;

    let (sig, recovery_id) = signing_key.sign_prehash_recoverable(&hash)?;
    let mut sig_bytes = sig.to_bytes().to_vec(); // r(32) + s(32)
    sig_bytes.push(recovery_id.to_byte()); // recovery id

    // Крок 3: додаємо підпис до транзакції і broadcast
    let mut signed_tx = create_resp.clone();
    signed_tx["signature"] = serde_json::json!([hex::encode(&sig_bytes)]);

    let broadcast_resp: serde_json::Value = client
        .post(format!("{NILE}/wallet/broadcasttransaction"))
        .json(&signed_tx)
        .send()
        .await?
        .json()
        .await?;

    if broadcast_resp["result"].as_bool() != Some(true) {
        return Err(anyhow!("{}", broadcast_resp));
    }

    Ok(tx_id_hex.to_string())
}

// Tron адреси внутрішньо зберігаються у hex форматі (41xxxxxx)
fn to_hex_address_from_base58(addr: &str) -> Result<String> {
    let bytes = bs58::decode(addr).into_vec()?;
    Ok(hex::encode(&bytes[..bytes.len() - 4])) // прибираємо checksum (останні 4 байти)
}

// Отримуємо адресу власника з приватного ключа
fn to_hex_address(private_key_hex: &str) -> Result<String> {
    use k256::ecdsa::VerifyingKey;
    use sha3::{Digest as _, Keccak256};

    let key_bytes = hex::decode(private_key_hex)?;
    let signing_key = SigningKey::from_slice(&key_bytes)?;
    let verifying_key = VerifyingKey::from(&signing_key);

    // Некомпресована публічна точка (65 байт: 04 || x || y)
    let pubkey_bytes = verifying_key.to_encoded_point(false);
    let pubkey_uncompressed = pubkey_bytes.as_bytes();

    // Ethereum-стиль адреса: keccak256 останніх 64 байт pubkey → останні 20 байт
    let hash = Keccak256::digest(&pubkey_uncompressed[1..]);
    let eth_addr = &hash[12..]; // останні 20 байт

    // Tron адреса = 0x41 prefix + eth_addr
    let mut tron = vec![0x41u8];
    tron.extend_from_slice(eth_addr);
    Ok(hex::encode(&tron))
}

// Повертає (private_key_hex, address_base58check)
pub fn keygen() -> (String, String) {
    use k256::ecdsa::SigningKey;
    use sha2::{Digest, Sha256};
    use sha3::Keccak256;

    let key_bytes: [u8; 32] = rand::random();
    let signing_key = SigningKey::from_slice(&key_bytes).expect("valid key");

    let pubkey = signing_key.verifying_key().to_encoded_point(false);
    let hash = Keccak256::digest(&pubkey.as_bytes()[1..]);

    // Tron адреса: 0x41 + останні 20 байт keccak256(pubkey)
    let mut addr = vec![0x41u8];
    addr.extend_from_slice(&hash[12..]);

    // Base58Check: addr + sha256(sha256(addr))[..4]
    let checksum = Sha256::digest(Sha256::digest(&addr));
    addr.extend_from_slice(&checksum[..4]);

    (hex::encode(key_bytes), bs58::encode(&addr).into_string())
}
