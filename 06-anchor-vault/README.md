# Anchor Vault

Solana смарт-контракт написаний на Rust + Anchor. Дозволяє зберігати SOL у персональному vault через PDA-акаунти.

## Що реалізовано

- `initialize` — створює vault для користувача (два PDA: state + vault)
- `deposit(amount)` — кладе SOL у vault
- `withdraw(amount)` — виводить SOL назад власнику

## Програма на devnet

```
https://explorer.solana.com/address/6doU3yt22cojkmVxdWeNTWSxYiRDXEjQueBJkRf1dbQ5?cluster=devnet
```

## Залежності

| Інструмент | Версія |
|---|---|
| Rust | 1.89.0 |
| Anchor CLI | 1.0.1 |
| Solana CLI | 3.1.14 |
| Node.js | 24+ |
| Yarn | будь-яка |

**Rust залежності** (`programs/anchor_vault/Cargo.toml`):
- `anchor-lang` 1.0.1 — фреймворк для Solana програм
- `litesvm` 0.10.0 — локальний Solana для тестів
- `solana-*` 3.x — клієнтські крейти

**JS залежності** (`package.json`):
- `@anchor-lang/core` — TypeScript клієнт для Anchor програм
- `ts-node`, `typescript` — запуск TypeScript скриптів

## Встановлення

```bash
# Rust, Solana CLI, Anchor CLI, Node.js, Yarn — все одною командою
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash

# Перевірка
rustc --version && solana --version && anchor --version && node --version && yarn --version

# JS залежності проєкту
yarn install
```

## Команди

```bash
# Збірка програми
anchor build

# Запуск тестів (LiteSVM — без devnet)
cargo test -p anchor_vault

# Синхронізація Program ID
anchor keys sync

# Деплой на devnet
solana config set --url devnet
solana airdrop 2
anchor program deploy --provider.cluster devnet

# Інтеракція з програмою на devnet
npx ts-node scripts/interact.ts
```

## Структура

```
├── programs/anchor_vault/
│   ├── src/
│   │   ├── lib.rs              # точка входу, оголошення інструкцій
│   │   ├── state.rs            # VaultState структура
│   │   ├── instructions/
│   │   │   ├── initialize.rs   # створення vault
│   │   │   ├── deposit.rs      # депозит SOL
│   │   │   └── withdraw.rs     # виведення SOL
│   │   ├── constants.rs
│   │   └── error.rs
│   └── tests/
│       ├── test_initialize.rs
│       ├── test_deposit.rs
│       └── test_withdraw.rs
├── scripts/
│   └── interact.ts             # скрипт для взаємодії з devnet
└── target/
    ├── deploy/anchor_vault.so  # скомпільована програма
    ├── idl/anchor_vault.json   # IDL для клієнтів
    └── types/anchor_vault.ts   # TypeScript типи
```

## Що вивчено

**Solana концепції:**
- **Accounts** — все на Solana є акаунтом (код, дані, гаманці)
- **PDA (Program Derived Address)** — акаунт без приватного ключа, яким керує програма через seeds + bump
- **Rent exemption** — акаунт повинен мати мінімальний баланс щоб існувати
- **CPI (Cross-Program Invocation)** — виклик однієї програми з іншої (наприклад System Program для переказу SOL)
- **Signer seeds** — як PDA підписує транзакції через `invoke_signed`

**Anchor:**
- `#[program]` — макрос точки входу
- `#[derive(Accounts)]` — декларативна валідація акаунтів
- `init`, `mut`, `seeds`, `bump` — constraints для акаунтів
- `ctx.bumps` — автоматичне збереження bump під час деривації PDA
- `CpiContext::new` та `CpiContext::new_with_signer` — контекст для CPI викликів

**Тестування:**
- LiteSVM — локальний Solana рантайм для unit тестів без devnet
- `include_bytes!` — вбудовування `.so` файлу в тест під час компіляції
