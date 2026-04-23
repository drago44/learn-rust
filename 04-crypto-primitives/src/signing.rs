use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};

// Підписує довільне повідомлення приватним ключем (ECDSA + SHA-256 під капотом).
// Підпис — 64 байти (r + s), унікальний для кожного повідомлення і ключа.
pub fn sign_message(signing_key: &SigningKey, message: &str) -> Signature {
    signing_key.sign(message.as_bytes())
}

// Верифікує підпис публічним ключем. Повертає true лише якщо:
// — підпис створений саме цим приватним ключем
// — повідомлення не було змінено після підписування
pub fn verify_message(verifying_key: &VerifyingKey, message: &str, signature: &Signature) -> bool {
    verifying_key.verify(message.as_bytes(), signature).is_ok()
}
