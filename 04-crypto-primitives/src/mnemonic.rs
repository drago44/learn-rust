use bip39::{Language, Mnemonic};
use rand::Rng;

// Генерує 16 байтів ентропії (128 біт) → 12 слів мнемоніки.
// Ентропія береться від ОС через криптографічно безпечний RNG.
pub fn generate_mnemonic() -> Mnemonic {
    let mut entropy = [0u8; 16]; // 16 байт = 128 біт = 12 слів
    rand::rng().fill_bytes(&mut entropy);
    Mnemonic::from_entropy_in(Language::English, &entropy).unwrap()
}

// Відновлює мнемоніку з рядка (для імпорту існуючого гаманця).
pub fn mnemonic_from_phrase(phrase: &str) -> Result<Mnemonic, bip39::Error> {
    Mnemonic::parse(phrase)
}
