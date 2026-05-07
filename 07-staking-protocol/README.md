# Staking Protocol (Solana / Anchor)

> **Застереження.** Це **навчальний / портфоліо-проєкт**, написаний для демонстрації механіки on-chain стейкінгу та паттернів безпеки в Anchor.
>
> - **Не аудитований.** Жодної формальної перевірки не проводилось. Можливі помилки в логіці, обмеженнях акаунтів, арифметиці.
> - **Не задеплоєний** на жоден публічний кластер (devnet/testnet/mainnet). Працює лише локально через LiteSVM-тести.
> - **Не для реальних коштів.** Не використовувати з токенами, що мають економічну вартість, без повного аудиту і ревізії економічної моделі.
> - **Економічна модель умовна.** `reward_rate`, supply reward-токена, поповнення `reward_vault` — все на розсуд адміна. Програма не моделює стійкість пулу.
> - **Адмін має значні повноваження** — може змінити `reward_rate` будь-коли. У продакшні цю роль зазвичай виконує мульти-сиг або DAO-governance, не один ключ.
>
> Якщо хочеш форкнути для реального продукту — мінімум: незалежний security-аудит, мульти-сиг для admin authority, формальна верифікація rewards-математики, обмеження на `reward_rate` (max-cap), захист від rug-pull (timelock на адмін-операції).

---

On-chain staking pool у стилі Synthetix `StakingRewards`: користувачі вносять стейк-токени, безперервно нараховують rewards у часі, виводять стейк через 7-денний cooldown.

Ключові властивості:

- **Synthetix-style accumulator** — `reward_per_token_stored` росте лінійно з часу та масштабу пулу. O(1) на всіх операціях, незалежно від кількості стейкерів.
- **Settlement-before-mutation** — будь-яка зміна `amount_staked` спочатку фіксує нараховане у `pending_rewards`, тож зміна позиції не «з'їдає» накопичене.
- **Окремий `reward_vault`** — rewards виплачуються з ізольованого vault під реальним PDA. Стейк (principal) і rewards ніколи не змішуються.
- **PDA-as-authority** — обидва vault'и керуються виключно програмою; адмін не може вивести користувацький стейк.
- **`checked_*` арифметика всюди**, з ескалацією до `MathOverflow`.
- **Anchor events** на всі мутаційні інструкції — для off-chain індексаторів.

---

## Архітектура

```
                      ┌──────────────────────┐
                      │     StakingPool      │   (PDA: ["pool", stake_mint])
                      │  authority           │
                      │  stake_mint          │
                      │  reward_mint         │
                      │  vault               │
                      │  reward_vault        │
                      │  total_staked        │
                      │  reward_rate         │  ← rewards/sec у найменших одиницях
                      │  reward_per_token_   │  ← глобальний accumulator (×1e9 для точності)
                      │       stored         │
                      │  last_update_time    │
                      │  bump                │
                      └──────────┬───────────┘
                                 │ authority (PDA)
                  ┌──────────────┼──────────────┐
                  │              │              │
            ┌─────▼────┐   ┌─────▼─────┐  ┌─────▼─────┐
            │  vault   │   │  reward_  │  │   user    │  (PDA: ["user", pool, user])
            │          │   │   vault   │  │  stakes   │  ┌─ owner
            │  stake_  │   │  reward_  │  │           │  ├─ amount_staked
            │  mint    │   │   mint    │  │  per      │  ├─ reward_debt  (індекс на момент settlement)
            └──────────┘   └───────────┘  │  user     │  ├─ pending_rewards
                                          └───────────┘  └─ bump
                                                              │
                                                              │ один user → багато
                                                              ▼
                                          ┌─────────────────────────┐
                                          │     UnstakeRequest      │  (PDA: ["unstake", pool, owner, request_time])
                                          │  owner                  │
                                          │  amount                 │
                                          │  request_time (nonce)   │
                                          │  created_at (clock)     │ ← від нього тікає COOLDOWN
                                          │  bump                   │
                                          └─────────────────────────┘
```

### Дизайн `request_time` vs `created_at`

`request_time` — параметр клієнта, потрібен лише для унікальності PDA (інакше юзер не міг би мати кілька паралельних `UnstakeRequest`). `created_at` пише сама програма з `Clock::get()` і саме від нього тікає 7-денний cooldown. Якби cooldown залежав від клієнтського параметра, атакер передавав би `request_time = now - 7 days` і клеймив одразу.

---

## Формула rewards (Synthetix-style)

Глобальний індекс пулу оновлюється при будь-якій взаємодії:

```
elapsed = now − last_update_time
reward_per_token_stored += elapsed × reward_rate × 1e9 / total_staked
last_update_time = now
```

> Множник `1e9` — це precision shift, щоб не втрачати дробові частини при цілочисельному діленні.

Нараховане для конкретного юзера:

```
earned = (reward_per_token_stored − user.reward_debt) × user.amount_staked / 1e9
```

Settlement (виконується перед будь-якою зміною `user.amount_staked`):

```
user.pending_rewards += earned
user.reward_debt      = reward_per_token_stored
```

Це **O(1)** на одну операцію — всі стейкери оновлюються неявно через спільний `reward_per_token_stored`. Без такого індексу довелось би ітерувати всіх стейкерів при зміні rate, що не масштабується.

---

## Інструкції

| Ім'я                 | Хто викликає | Що робить |
|----------------------|---------------|-----------|
| `initialize_pool`    | admin         | створює `StakingPool`, `vault`, `reward_vault` |
| `stake(amount)`      | user          | settle → CPI transfer → `total_staked +=`, `user.amount_staked +=` |
| `unstake(amount, request_time)` | user | settle → зменшення позиції → створення `UnstakeRequest` (токени поки залишаються у vault) |
| `claim`              | user          | перевірка cooldown → CPI transfer з vault → закриття `UnstakeRequest` |
| `harvest`            | user          | settle → CPI transfer rewards з `reward_vault` → обнуляє `pending_rewards` |
| `update_reward_rate(new_rate)` | admin | settle → змінює `reward_rate` |

Усі мутаційні інструкції емітять `#[event]` для off-chain парсингу.

---

## Безпека

Чек-лист:

- [x] `checked_add` / `checked_mul` / `checked_div` + `u128` всюди → `MathOverflow`
- [x] `amount > 0` guard на `stake` та `unstake`
- [x] `total_staked > 0` перед діленням у формулі
- [x] `has_one = authority` на `update_reward_rate`
- [x] `has_one = owner` на `claim` (поверх seeds-перевірки)
- [x] Bump validation (`bump = X.bump`) при ре-використанні PDA
- [x] Mint+vault фіксовані через `address = pool.vault` / `address = pool.stake_mint`
- [x] PDA-signed CPI на `harvest` / `claim` (authority = pool, не user)
- [x] `pending_rewards = 0` ВЕРОЛО CPI у `harvest` (re-entrancy hygiene)
- [x] `request_time` (клієнт) розв'язано від `created_at` (cooldown source-of-truth)
- [x] Settlement ПЕРЕД будь-якою зміною `amount_staked` — нараховане ніколи не губиться

---

## Запуск тестів

```bash
# збираємо програму
anchor build

# повний прогон usage scenarios через LiteSVM
cargo test -p staking_protocol

# окремий файл
cargo test -p staking_protocol --test test_harvest
```

Покриття:

- `test_initialize` — happy path + повторна ініціалізація → fail
- `test_stake` — баланси оновлюються, токени фізично у vault, `stake(0)` → fail
- `test_unstake` — зменшення `total_staked`, кілька паралельних запитів, `unstake(>balance)` → fail
- `test_harvest` — нуль одразу після стейку, accrual через `set_sysvar::<Clock>`, double harvest, two-stakers proportional, division-by-zero guard
- `test_claim` — до cooldown → fail, після cooldown → токени + closed account, чужий claim → fail
- `test_update_reward_rate` — admin OK, чужий → fail, settlement за старою ставкою

---

## Деплой на devnet

```bash
# 1. Перевір cluster
solana config set --url devnet
solana airdrop 2

# 2. Згенеруй keypair програми (один раз)
anchor keys list                         # покаже поточний program ID
# або заміни ключ:
solana-keygen new -o target/deploy/staking_protocol-keypair.json

# 3. Підстав ID у lib.rs (declare_id!) та Anchor.toml (programs.devnet.staking_protocol)
anchor build
anchor keys sync                         # синхронізує declare_id! зі Anchor.toml

# 4. Деплой
anchor deploy --provider.cluster devnet

# 5. Лог-стрім (в окремому терміналі)
solana logs $(anchor keys list | awk '/staking_protocol/ {print $2}')
```

Smoke-перевірка після деплою (приклад):

```bash
# Створюємо два mint'и (stake + reward), ініціалізуємо пул, стейкаємо, чекаємо, harvest.
# Команди — через окремий клієнтський бінарник (out of scope).
```

---

## Структура коду

```
programs/staking_protocol/src/
├── lib.rs                         ← entry: declare_id! + #[program] handlers
├── constants.rs                   ← COOLDOWN_SECONDS
├── state.rs                       ← StakingPool, UserStake, UnstakeRequest + SIZE
├── error.rs                       ← StakingError enum
├── events.rs                      ← #[event] структури
├── helpers.rs                     ← update_reward_per_token, earned, accrue_user_rewards
└── instructions/
    ├── initialize.rs
    ├── stake.rs
    ├── unstake.rs
    ├── claim.rs
    ├── harvest.rs
    └── update_reward_rate.rs

programs/staking_protocol/tests/   ← LiteSVM integration tests (Rust, не TS)
```
