use crate::hashing::hash_keccak256;
use k256::SecretKey;
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand::Rng;

// Ethereum адреса — це останні 20 байтів keccak256 від публічного ключа.
// Публічний ключ в нестиснутому форматі починається з байта 04 (маркер),
// тому пропускаємо перший байт і хешуємо лише 64 байти координат X і Y.
pub fn eth_address(verifying_key: &VerifyingKey) -> String {
    let pub_key = verifying_key.to_encoded_point(false); // false = нестиснутий (04 + X + Y)
    let pub_bytes = &pub_key.as_bytes()[1..]; // прибираємо маркер 04
    let hash = hash_keccak256(pub_bytes);
    format!("0x{}", hex::encode(&hash[12..])) // беремо байти 12..32 (останні 20)
}

// Генерує нову пару ключів secp256k1 — та сама крива що використовують ETH і BTC.
// Приватний ключ — 32 випадкових байти від ОС.
// Публічний ключ виводиться з приватного через множення на генераторну точку кривої.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut rng = rand::rng();
    let mut secret_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_bytes); // криптографічно безпечний RNG від ОС

    let secret_key = SecretKey::from_bytes(&secret_bytes.into()).unwrap();
    let signing_key = SigningKey::from(&secret_key);
    let verifying_key = VerifyingKey::from(&signing_key);

    println!("Private key: {}", hex::encode(secret_key.to_bytes()));
    println!(
        "Public key:  {}",
        hex::encode(verifying_key.to_encoded_point(false).as_bytes())
    );
    println!("ETH address: {}", eth_address(&verifying_key));

    (signing_key, verifying_key)
}
