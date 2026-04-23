mod hashing;
mod keys;

fn main() {
    let msg = "hello crypto";
    println!("Input:  {}", msg);
    println!("SHA256: {}", hashing::hash_sha256(msg));
    println!(
        "Keccak: {}",
        hex::encode(hashing::hash_keccak256(msg.as_bytes()))
    );
    keys::generate_keypair();
}
