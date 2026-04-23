// Публічні модулі крейту. Кожен модуль — окрема зона відповідальності.
pub mod hashing; // SHA-256, BLAKE3, Keccak-256
pub mod hdwallet; // HD гаманці (BIP32, BIP44)
pub mod keys; // secp256k1 (ETH/BTC) та ed25519 (Solana) ключі та адреси
pub mod mnemonic; // Мнемонічні фрази (BIP39)
pub mod signing; // Підписування та верифікація ECDSA
