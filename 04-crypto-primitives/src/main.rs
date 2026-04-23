use k256::SecretKey;
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand::Rng;
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn hash_keccak256(input: &str) -> String {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(input.as_bytes());
    hasher.finalize(&mut output);
    hex::encode(output)
}

fn generate_keypair() {
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
}

fn main() {
    let msg = "hello crypto";
    println!("Input:  {}", msg);
    println!("SHA256: {}", hash_sha256(msg));
    println!("Keccak: {}", hash_keccak256(msg));
    generate_keypair();
}
