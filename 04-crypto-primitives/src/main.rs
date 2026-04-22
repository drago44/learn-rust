use sha2::{Digest, Sha256};

fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn main() {
    let msg = "hello crypto";
    println!("Input:  {}", msg);
    println!("SHA256: {}", hash_sha256(msg));
}
