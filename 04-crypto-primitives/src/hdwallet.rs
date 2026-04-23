use coins_bip32::path::DerivationPath;
use coins_bip32::prelude::*;
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
