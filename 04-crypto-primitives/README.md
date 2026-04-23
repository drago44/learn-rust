# Місяць 4 ✅ — Crypto primitives

**Що вивчаємо:**
##### ✅ Хешування — SHA256, BLAKE3, keccak256
##### ✅ Асиметрична криптографія — `secp256k1` (BTC/ETH)
##### ✅ Асиметрична криптографія — `ed25519` (Solana)
##### ✅ Підписування транзакцій — sign/verify
##### ✅ BIP39 — мнемонічні фрази (seed phrase)
##### ✅ BIP32 — HD гаманці, деривація ключів
##### ✅ BIP44 — шляхи деривації (`m/44'/60'/0'/0/0`)
##### ✅ Ethereum адреси — з публічного ключа
##### ✅ Bitcoin адреси — P2PKH, P2WPKH
##### ✅ EIP-55 — checksum для ETH адрес
##### ✅ Solana деривація з мнемоніки — `m/44'/501'/0'/0'`
##### ✅ Schnorr/P2TR — адреса Bitcoin Taproot (`bc1p...`, Bech32m, x-only pubkey)
##### ✅ Multisig M-of-N — P2SH адреса (`3...`, OP_M + pubkeys + OP_N + OP_CHECKMULTISIG)

**Проєкт: `04-crypto-primitives/`**
##### ✅ Генерація мнемоніки (12 слів)
##### ✅ Деривація ключів для ETH (BIP44)
##### ✅ Генерація BTC адрес — P2PKH, P2WPKH, P2TR
##### ✅ Підписування повідомлення та верифікація
##### ✅ Генерація Ethereum адреси з приватного ключа
##### ✅ Генерація Solana адреси з публічного ключа
##### ✅ CLI: `wallet new`, `wallet derive`, `wallet sign`, `wallet multisig`, `wallet demo`

**CLI команди:**
```bash
cargo run -- new                        # нова мнемоніка + адреси ETH/BTC(P2PKH,P2WPKH,P2TR)/SOL
cargo run -- derive --index 1           # деривація адрес по індексу
cargo run -- sign --message "hello"     # підписати повідомлення
cargo run -- multisig --m 2 --n 3      # 2-of-3 мультисиг адреса (P2SH)
cargo run -- demo                       # демо всіх криптографічних примітивів
```

**Архітектура:**
```
src/
  hashing.rs   ← SHA256, BLAKE3, keccak256
  keys.rs      ← ETH/BTC/SOL адреси з публічного ключа
  mnemonic.rs  ← генерація та парсинг BIP39 мнемоніки
  hdwallet.rs  ← деривація ключів (BIP32/BIP44)
  signing.rs   ← sign/verify через secp256k1
  demo.rs      ← демонстрація всіх примітивів
  main.rs      ← CLI через clap
```
