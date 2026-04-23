use crate::hashing::{hash_keccak256, hash_sha256_bytes};
use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey};
use k256::SecretKey;
use k256::ecdsa::{SigningKey as Secp256k1SigningKey, VerifyingKey as Secp256k1VerifyingKey};
use rand::Rng;
use ripemd::{Digest as RipemdDigest, Ripemd160};

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

    println!(
        "  secp256k1 private: {}",
        hex::encode(secret_key.to_bytes())
    );
    println!(
        "  secp256k1 public:  {}",
        hex::encode(verifying_key.to_encoded_point(false).as_bytes())
    );
    println!("  (ETH і BTC використовують ці самі ключі, різні лише адреси)");

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

// EIP-55 — checksum адреса: регістр кожної hex-букви визначається keccak256 хешем.
// Стандарт де-факто — MetaMask і Etherscan завжди показують адреси в такому форматі.
// Якщо переплутати регістр букви — гаманець визначить адресу як невалідну.
pub fn eth_address_checksum(verifying_key: &Secp256k1VerifyingKey) -> String {
    let addr = eth_address(verifying_key);
    let hex = &addr[2..]; // прибираємо "0x"
    let hash = hash_keccak256(hex.as_bytes());

    let checksummed: String = hex
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_digit() {
                c
            } else if (hash[i / 2] >> (if i % 2 == 0 { 4 } else { 0 }) & 0xf) >= 8 {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();

    format!("0x{}", checksummed)
}

// =============================================================================
// Bitcoin адреси (secp256k1)
// =============================================================================

// hash160 = RIPEMD160(SHA256(pubkey)) — стандартна операція в Bitcoin.
// Використовується в обох форматах адрес.
fn hash160(input: &[u8]) -> [u8; 20] {
    let sha256 = hash_sha256_bytes(input);
    let mut hasher = Ripemd160::new();
    hasher.update(sha256);
    hasher.finalize().into()
}

// P2PKH (legacy) — починається з '1'.
// Формат: Base58Check( 0x00 + hash160(pubkey) )
pub fn btc_address_p2pkh(verifying_key: &Secp256k1VerifyingKey) -> String {
    // Стиснутий ключ (33 байти: 02/03 + X) — стандарт для сучасних BTC гаманців
    let pub_bytes = verifying_key.to_encoded_point(true).as_bytes().to_vec();
    let h160 = hash160(&pub_bytes);

    // Версійний байт 0x00 = mainnet P2PKH
    let mut payload = vec![0x00u8];
    payload.extend_from_slice(&h160);
    bs58::encode(payload).with_check().into_string()
}

// P2WPKH (SegWit) — починається з 'bc1q'.
// Формат: Bech32( "bc", witness_v0, hash160(pubkey) )
pub fn btc_address_p2wpkh(verifying_key: &Secp256k1VerifyingKey) -> String {
    use bech32::segwit;
    let pub_bytes = verifying_key.to_encoded_point(true).as_bytes().to_vec();
    let h160 = hash160(&pub_bytes);
    // witness version 0 + hash160 → bech32 з hrp "bc" (mainnet)
    segwit::encode_v0(bech32::hrp::BC, &h160).unwrap()
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

    println!("  ed25519 private: {}", hex::encode(signing_key.to_bytes()));
    println!(
        "  ed25519 public:  {}",
        hex::encode(verifying_key.as_bytes())
    );

    (signing_key, verifying_key)
}

// Solana адреса — просто base58 від публічного ключа (32 байти).
// Ніякого хешування на відміну від ETH.
pub fn solana_address(verifying_key: &Ed25519VerifyingKey) -> String {
    bs58::encode(verifying_key.as_bytes()).into_string()
}
