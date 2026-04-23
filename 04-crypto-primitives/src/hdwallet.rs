use coins_bip32::path::DerivationPath;
use coins_bip32::prelude::*;
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use ed25519_dalek_bip32::{DerivationPath as SolDerivationPath, ExtendedSigningKey};
use k256::ecdsa::{SigningKey, VerifyingKey};

// Деривує пару ключів з seed по BIP44 шляху.
// m/44'/60'/0'/0/index — стандартний ETH шлях (той самий що MetaMask).
// Апостроф (') означає "hardened" деривацію — дочірній ключ не можна
// вирахувати знаючи лише публічний батьківський ключ.
pub fn derive_eth_keypair(seed: &[u8], index: u32) -> (SigningKey, VerifyingKey) {
    let path = format!("m/44'/60'/0'/0/{}", index);

    let xpriv = XPriv::root_from_seed(seed, None).unwrap();
    let child = xpriv
        .derive_path(path.parse::<DerivationPath>().unwrap())
        .unwrap();

    // XPriv реалізує AsRef<k256::ecdsa::SigningKey>
    let signing_key: SigningKey = AsRef::<SigningKey>::as_ref(&child).clone();
    let verifying_key = VerifyingKey::from(&signing_key);

    (signing_key, verifying_key)
}

// Деривує Solana ключ з seed по SLIP-0010 шляху.
// m/44'/501'/0'/0' — стандартний SOL шлях (Phantom, Solflare).
// Всі компоненти hardened (') бо ed25519 не підтримує non-hardened деривацію.
pub fn derive_sol_keypair(seed: &[u8], index: u32) -> Ed25519SigningKey {
    let path: SolDerivationPath = format!("m/44'/501'/{}'/{}'", index, 0).parse().unwrap();

    let xkey = ExtendedSigningKey::from_seed(seed).unwrap();
    let derived = xkey.derive(&path).unwrap();
    derived.signing_key
}
