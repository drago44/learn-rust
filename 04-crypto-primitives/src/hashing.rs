use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

// SHA-256 — стандартний криптографічний хеш. Використовується в Bitcoin,
// TLS, підписах. Повертає hex-рядок для зручного виводу.
pub fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// Keccak-256 — варіант SHA-3, який обрав Ethereum (не стандартний SHA3-256!).
// Повертає сирі байти [u8; 32], щоб їх можна було переиспользовувати
// без зайвого перетворення (наприклад, для деривації ETH адреси).
pub fn hash_keccak256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(input);
    hasher.finalize(&mut output);
    output
}
