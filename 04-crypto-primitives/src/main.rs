use crypto_primitives::{hashing, hdwallet, keys, mnemonic, signing};

fn main() {
    let msg = "hello crypto";

    // --- Хешування ---
    println!("Input:  {}", msg);
    println!("SHA256: {}", hashing::hash_sha256(msg));
    println!("BLAKE3: {}", hashing::hash_blake3(msg.as_bytes()));
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

    // --- Мнемоніка (BIP39) ---
    let mnemonic = mnemonic::generate_mnemonic();
    println!("\nMnemonic: {}", mnemonic);
    let seed = mnemonic.to_seed(""); // "" = без додаткового пароля
    println!("Seed:     {}", hex::encode(&seed[..8]));

    // --- HD гаманець (BIP32/BIP44) ---
    // З однієї мнемоніки — необмежена кількість адрес, як у MetaMask
    let (_, hd_verifying0) = hdwallet::derive_eth_keypair(&seed, 0);
    let (_, hd_verifying1) = hdwallet::derive_eth_keypair(&seed, 1);
    println!("ETH[0]:   {}", keys::eth_address(&hd_verifying0));
    println!("ETH[1]:   {}", keys::eth_address(&hd_verifying1));
}
