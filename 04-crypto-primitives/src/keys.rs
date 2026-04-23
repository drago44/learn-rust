use crate::hashing::hash_keccak256;
use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey};
use k256::SecretKey;
use k256::ecdsa::{SigningKey as Secp256k1SigningKey, VerifyingKey as Secp256k1VerifyingKey};
use rand::Rng;

// =============================================================================
// secp256k1 — крива для ETH і BTC
// =============================================================================

// Генерує нову пару ключів secp256k1.
// Приватний ключ — 32 випадкових байти від ОС.
// Публічний ключ виводиться з приватного через множення на генераторну точку кривої.
pub fn generate_secp256k1_keypair() -> (Secp256k1SigningKey, Secp256k1VerifyingKey) {
    let mut secret_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut secret_bytes);

    let secret_key = SecretKey::from_bytes(&secret_bytes.into()).unwrap();
    let signing_key = Secp256k1SigningKey::from(&secret_key);
    let verifying_key = Secp256k1VerifyingKey::from(&signing_key);

    println!("ETH private: {}", hex::encode(secret_key.to_bytes()));
    println!(
        "ETH public:  {}",
        hex::encode(verifying_key.to_encoded_point(false).as_bytes())
    );

    (signing_key, verifying_key)
}

// Ethereum адреса — останні 20 байтів keccak256 від публічного ключа.
// Публічний ключ в нестиснутому форматі починається з байта 04 (маркер),
// тому пропускаємо перший байт і хешуємо лише 64 байти координат X і Y.
pub fn eth_address(verifying_key: &Secp256k1VerifyingKey) -> String {
    let pub_key = verifying_key.to_encoded_point(false); // false = нестиснутий (04 + X + Y)
    let pub_bytes = &pub_key.as_bytes()[1..]; // прибираємо маркер 04
    let hash = hash_keccak256(pub_bytes);
    format!("0x{}", hex::encode(&hash[12..])) // беремо байти 12..32 (останні 20)
}

// =============================================================================
// ed25519 — крива для Solana (і SSH, GPG, Monero)
// =============================================================================

// Генерує нову пару ключів ed25519.
// Швидша за secp256k1, підписи менші, математика простіша.
pub fn generate_ed25519_keypair() -> (Ed25519SigningKey, Ed25519VerifyingKey) {
    let mut secret_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut secret_bytes);

    let signing_key = Ed25519SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    println!("SOL private: {}", hex::encode(signing_key.to_bytes()));
    println!("SOL public:  {}", hex::encode(verifying_key.as_bytes()));

    (signing_key, verifying_key)
}

// Solana адреса — просто base58 від публічного ключа (32 байти).
// Ніякого хешування на відміну від ETH.
pub fn solana_address(verifying_key: &Ed25519VerifyingKey) -> String {
    bs58::encode(verifying_key.as_bytes()).into_string()
}
