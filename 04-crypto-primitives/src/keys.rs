use crate::hashing::hash_keccak256;
use k256::SecretKey;
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand::Rng;

pub fn eth_address(verifying_key: &VerifyingKey) -> String {
    let pub_key = verifying_key.to_encoded_point(false);
    let pub_bytes = &pub_key.as_bytes()[1..]; // прибираємо 04
    let hash = hash_keccak256(pub_bytes);
    format!("0x{}", hex::encode(&hash[12..])) // останні 20 байтів
}

pub fn generate_keypair() {
    let mut rng = rand::rng();
    let mut secret_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_bytes);

    let secret_key = SecretKey::from_bytes(&secret_bytes.into()).unwrap();
    let signing_key = SigningKey::from(&secret_key);
    let verifying_key = VerifyingKey::from(&signing_key);

    println!("Private key: {}", hex::encode(secret_key.to_bytes()));
    println!(
        "Public key:  {}",
        hex::encode(verifying_key.to_encoded_point(false).as_bytes())
    );
    println!("ETH address: {}", eth_address(&verifying_key));
}
