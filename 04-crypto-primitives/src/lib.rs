// Публічні модулі крейту. Кожен модуль — окрема зона відповідальності.
pub mod hashing; // SHA-256 і Keccak-256
pub mod keys; // Генерація ключів secp256k1, Ethereum адреса
pub mod signing; // Підписування та верифікація ECDSA
