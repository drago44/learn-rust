# 05-multichain

Multi-chain CLI для читання балансів, транзакцій, відправки та генерації гаманців.
Підтримувані мережі: **Bitcoin**, **Ethereum**, **Solana**, **Tron**.

---

## Архітектура

```
src/
├── main.rs      — точка входу, маршрутизація команд
├── cli.rs       — ChainCmd (clap) + QueryCmd (display routing)
├── display.rs   — print_balance / print_txs через AsyncFn
├── types.rs     — Balance, Tx (спільні типи)
├── btc.rs       — Mempool.space API
├── eth.rs       — alloy + Ethplorer + WebSocket
├── sol.rs       — JSON-RPC напряму + ed25519 підпис
└── trx.rs       — TronGrid REST API + secp256k1 підпис
```

---

## Команди

### balance — поточний баланс адреси

```bash
cargo run -- btc balance 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
cargo run -- eth balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
cargo run -- sol balance 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
cargo run -- trx balance TCrSPhg8ERaeu3mzNesq92TP4fHyjoKWNh
```

### txs — останні 10 транзакцій

```bash
cargo run -- btc txs 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
cargo run -- eth txs 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
cargo run -- sol txs 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
cargo run -- trx txs TCrSPhg8ERaeu3mzNesq92TP4fHyjoKWNh
```

### watch — підписка на зміни балансу (WebSocket, Ctrl+C для виходу)

```bash
cargo run -- btc watch 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
cargo run -- eth watch 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
cargo run -- sol watch 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
# TRX watch не підтримується
```

### keygen — згенерувати новий тестовий гаманець

```bash
cargo run -- eth keygen
cargo run -- sol keygen
cargo run -- trx keygen
# BTC keygen не підтримується
```

Виводить адресу, приватний ключ і готову команду для `send`.

### send — відправити токени (тестові мережі)

> **Мережі:** ETH → Sepolia, SOL → devnet, TRX → Nile testnet

```bash
# Ethereum (Sepolia)
ETH_PRIVATE_KEY=0x<hex> cargo run -- eth send <to_address> <amount_eth>

# Solana (devnet)
SOL_PRIVATE_KEY=<base58_64bytes> cargo run -- sol send <to_address> <amount_sol>

# Tron (Nile testnet)
TRX_PRIVATE_KEY=<hex> cargo run -- trx send <to_address> <amount_trx>

# BTC send не підтримується
```

---

## Що вивчалось

| Тема | Де використовується |
|---|---|
| UTXO модель | `btc.rs` — Mempool.space |
| Account модель | `eth.rs`, `sol.rs`, `trx.rs` |
| `alloy` v2 — Ethereum RPC + підпис | `eth.rs` |
| JSON-RPC 2.0 напряму | `sol.rs` |
| WebSocket підписки | `btc.rs`, `eth.rs`, `sol.rs` |
| secp256k1 (k256) — підпис + адреса | `eth.rs`, `trx.rs` |
| ed25519 + compact-u16 серіалізація | `sol.rs` — ручна побудова транзакції |
| Base58Check | `trx.rs`, `sol.rs` |
| `AsyncFn` trait (Rust 1.85) | `display.rs` — generic async callback |
| `TryFrom` для маршрутизації enum | `cli.rs` — QueryCmd / ChainCmd |
