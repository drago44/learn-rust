use crypto_primitives::{hashing, keys, mnemonic, signing};

fn main() {
    let msg = "hello crypto";

    // --- Хешування ---
    println!("Input:  {}", msg);
    println!("SHA256: {}", hashing::hash_sha256(msg));
    println!(
        "Keccak: {}",
        hex::encode(hashing::hash_keccak256(msg.as_bytes()))
    );

    // --- Ключі та адреса ---
    let (signing_key, verifying_key) = keys::generate_keypair();

    // --- Підписування ---
    let sig = signing::sign_message(&signing_key, msg);
    println!("Signature:   {}", hex::encode(sig.to_bytes()));
    println!(
        "Valid:       {}",
        signing::verify_message(&verifying_key, msg, &sig)
    );
    println!(
        "Tampered:    {}",
        signing::verify_message(&verifying_key, "evil", &sig)
    );

    // --- Мнемоніка ---
    let mnemonic = mnemonic::generate_mnemonic();
    println!("Mnemonic: {}", mnemonic);
    let seed = mnemonic.to_seed(""); // "" = без пароля
    println!("Seed:     {}", hex::encode(&seed[..8])); // перші 8 байт для читабельності
}
