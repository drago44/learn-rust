use crypto_primitives::{hashing, hdwallet, keys, mnemonic, signing};

fn main() {
    let msg = "hello crypto";

    // =========================================================================
    // # Хешування
    // =========================================================================
    println!("=== Хешування ===");
    println!("  Input:  {}", msg);
    println!("  SHA256: {}", hashing::hash_sha256(msg));
    println!("  BLAKE3: {}", hashing::hash_blake3(msg.as_bytes()));
    println!(
        "  Keccak: {}",
        hex::encode(hashing::hash_keccak256(msg.as_bytes()))
    );

    // =========================================================================
    // # Ключі
    // =========================================================================

    // ## secp256k1 — ETH / BTC
    println!("\n=== Ключі: secp256k1 (ETH/BTC) ===");
    let (signing_key, verifying_key) = keys::generate_secp256k1_keypair();
    println!("  ETH address:     {}", keys::eth_address(&verifying_key));
    println!(
        "  ETH checksum:    {}",
        keys::eth_address_checksum(&verifying_key)
    );
    println!(
        "  BTC P2PKH:       {}",
        keys::btc_address_p2pkh(&verifying_key)
    );
    println!(
        "  BTC P2WPKH:      {}",
        keys::btc_address_p2wpkh(&verifying_key)
    );

    // ## ed25519 — Solana
    println!("\n=== Ключі: ed25519 (Solana) ===");
    let (_sol_signing, sol_verifying) = keys::generate_ed25519_keypair();
    println!(
        "  SOL address:     {}",
        keys::solana_address(&sol_verifying)
    );

    // =========================================================================
    // # Підписування (secp256k1)
    // =========================================================================
    println!("\n=== Підписування ===");
    let sig = signing::sign_message(&signing_key, msg);
    println!("  Message:   {}", msg);
    println!("  Signature: {}", hex::encode(sig.to_bytes()));
    println!(
        "  Valid:     {}",
        signing::verify_message(&verifying_key, msg, &sig)
    );
    println!(
        "  Tampered:  {}",
        signing::verify_message(&verifying_key, "evil", &sig)
    );

    // =========================================================================
    // # HD гаманець (BIP39 → BIP32 → BIP44)
    // =========================================================================
    println!("\n=== HD Гаманець ===");
    let mnemonic = mnemonic::generate_mnemonic();
    println!("  Mnemonic: {}", mnemonic);
    let seed = mnemonic.to_seed("");
    println!("  Seed:     {}", hex::encode(&seed[..8]));
    let (_, hd_verifying0) = hdwallet::derive_eth_keypair(&seed, 0);
    let (_, hd_verifying1) = hdwallet::derive_eth_keypair(&seed, 1);
    println!("  ETH[0]:   {}", keys::eth_address(&hd_verifying0));
    println!("  ETH[1]:   {}", keys::eth_address(&hd_verifying1));
}
