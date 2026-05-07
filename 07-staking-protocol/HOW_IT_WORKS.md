# Як працює Staking Protocol

> **Застереження.** Це навчальний / портфоліо-проєкт. Не аудитований, не задеплоєний на публічні кластери, не призначений для роботи з реальними коштами. Деталі — у [README.md](./README.md#staking-protocol-solana--anchor).

## Загальна ідея

Це **DeFi-стейкінг**. Користувачі **блокують** один токен (stake-токен), за це їм нараховують **інший** токен (reward-токен) пропорційно тому, скільки і як довго стейк лежав у пулі. Як у банку депозит, але прозоро on-chain.

```
┌─────────┐  stake(100)   ┌──────┐                ┌──────────┐
│  USER   │──────────────▶│ POOL │  накопичує час │ rewards  │
│         │               │      │ ───────────────▶          │
│         │◀──────────────│      │  harvest()     │          │
└─────────┘   reward      └──────┘                └──────────┘
```

---

## Три акаунти (стан програми)

### 1. `StakingPool` — глобальний стан пулу
Один на пару `(stake_mint, reward_mint)`. Зберігає:
- ставку нарахувань (`reward_rate` — скільки reward-токенів роздається за секунду на ВЕСЬ пул);
- сумарний стейк всіх юзерів (`total_staked`);
- **глобальний accumulator** `reward_per_token_stored` — серце механіки;
- адресу адміна (`authority`).

### 2. `UserStake` — позиція одного юзера
По одному на пару `(pool, user)`. Зберігає скільки юзер застейкав і скільки rewards йому ще винні, але ще не виплатили.

### 3. `UnstakeRequest` — квиток на вивід
Створюється при спробі зняти стейк. Юзер не отримує токени одразу — є **7-денний cooldown**. Квиток фіксує `created_at` і суму. Через 7 днів юзер `claim`-ить його, і квиток **закривається** (rent повертається).

---

## Два vault'и (де лежать токени)

Це звичайні SPL token accounts, але їхній **authority** — сама програма (PDA). Юзер не може вивести токени напряму, тільки через інструкції.

- `vault` — куди йдуть **застейкані** токени (principal);
- `reward_vault` — звідки виплачуються **rewards**.

Принципал і rewards **ніколи не змішуються** — це базова безпека: навіть якщо в reward_vault скінчаться токени, principal у vault залишається недоторканим.

---

## Шість інструкцій

| # | Інструкція | Хто | Що робить |
|---|---|---|---|
| 1 | `initialize_pool(reward_rate)` | admin | створює пул + обидва vault'и |
| 2 | `stake(amount)` | user | переводить токени у vault, оновлює позицію |
| 3 | `unstake(amount, request_time)` | user | списує з позиції, створює `UnstakeRequest`. Токени ще у vault! |
| 4 | `claim` | user | через 7+ днів — забирає принципал з vault, закриває request |
| 5 | `harvest` | user | будь-коли — забирає накопичені rewards з reward_vault |
| 6 | `update_reward_rate(new_rate)` | admin | змінює швидкість нарахування |

---

## Серце системи: формула rewards

### Проблема

Як справедливо ділити rewards між десятками тисяч стейкерів, не ітеруючи їх щоразу? Якщо при кожній зміні ставки/часу проходити по всіх — ціна транзакції росте лінійно з кількістю користувачів. На Solana це швидко вб'є compute budget.

### Рішення (Synthetix-style)

Замість того щоб **зберігати окремо** rewards для кожного юзера, ми ведемо **один глобальний лічильник** `reward_per_token_stored` — скільки rewards припадало б на 1 умовний застейканий токен з моменту створення пулу.

Він росте лінійно з часом:

```
Δ = elapsed × reward_rate × 1e9 / total_staked
reward_per_token_stored += Δ
```

(`× 1e9` — це **precision shift**: ми працюємо з цілими числами, тож множимо на 1 млрд щоб не втрачати дробові частини при діленні).

Кожен юзер при стейку запам'ятовує **знімок** цього лічильника у поле `reward_debt`. Заробіток юзера в будь-який момент:

```
earned = (reward_per_token_stored − user.reward_debt) × user.amount_staked / 1e9
```

Це різниця між поточним глобальним лічильником і тим, що було при стейку, помножена на скільки токенів юзер тримає. **Все за O(1)**, незалежно від кількості стейкерів.

### Settlement — критичний момент

Перед **будь-якою** зміною `amount_staked` (stake, unstake) або `reward_rate` (admin update) ми викликаємо `accrue_user_rewards`:

1. Оновлюємо глобальний `reward_per_token_stored` (з врахуванням нової кількості часу і поточного `total_staked`).
2. Рахуємо `earned` для юзера і **скидаємо в `pending_rewards`**.
3. Підіймаємо `user.reward_debt = reward_per_token_stored`.

Без settlement-у: юзер застейкав 1000 токенів, час пройшов, він застейкає ще 500. Якщо одразу записати `reward_debt = поточний_лічильник`, то **всі попередні rewards губляться** — ми не зафіксували їх до того, як змінити позицію. Synthetix-паттерн розв'язує це через окреме поле `pending_rewards`.

---

## Cooldown: чому 7 днів і чому два поля

```rust
pub struct UnstakeRequest {
    pub request_time: i64,  // nonce клієнта (тільки в seeds PDA)
    pub created_at: i64,    // реальний on-chain час (cooldown від нього)
    ...
}
```

**Навіщо два поля?** Якби cooldown тікав від `request_time` (параметра клієнта), атакер міг би передати `request_time = now − 7 days` і клеймити одразу. Тому програма **сама** записує `created_at = Clock::get()` — це не підробити.

**Навіщо `request_time` тоді взагалі?** Він живе у seeds: `[b"unstake", pool, owner, request_time]`. Якщо у юзера буде кілька паралельних `UnstakeRequest`, кожен має бути окремим PDA — інакше другий `unstake` не зміг би створити акаунт. `request_time` — це просто **nonce** для унікальності.

Ціль cooldown — **запобігти масовому виходу** з пулу: якщо хтось маніпулює ціною reward-токена і хоче миттєво вивести стейк, у нього є 7 днів на роздуми.

---

## Критичні захисти

1. **PDA-as-authority** — обидва vault'и підписує тільки програма через `signer_seeds`. Юзер не може вкрасти стейк іншого юзера.

2. **`has_one`-перевірки** — `claim` має `has_one = owner`, `update_reward_rate` має `has_one = authority`. Anchor падає з `Unauthorized` ще до handler-у.

3. **Подвійний захист на claim** — seeds містять `owner.as_ref()`, тож чужий взагалі не зможе ре-дерайвнути той самий PDA. Плюс has_one. Подвійний.

4. **Settlement перед зміною rate** — `update_reward_rate` спочатку фіксує `reward_per_token_stored` за СТАРОЮ ставкою, тоді змінює rate. Інакше rewards за минулий період були б ретроактивно перерахованими за новою ставкою — або в плюс (юзери щасливі), або в мінус (атака).

5. **Re-entrancy hygiene** — у `harvest` ми обнуляємо `pending_rewards = 0` ДО CPI. На Solana це не критично як в Ethereum, але паттерн правильний.

6. **`checked_*` всюди** — арифметика через `checked_add` / `checked_mul` / `checked_div`, accumulator у `u128`. При переповненні — `MathOverflow`, не silent wrap.

---

## Типовий цикл життя стейкера

```
T+0:    user викликає stake(1000)
        → CPI transfer 1000 у vault
        → user.amount_staked = 1000
        → user.reward_debt = pool.rps  (snapshot)

T+30d:  user викликає harvest()
        → settle: pending_rewards = (pool.rps_now - reward_debt) × 1000 / 1e9
        → CPI transfer rewards з reward_vault
        → user.reward_debt = pool.rps_now (snapshot)
        → user.pending_rewards = 0

T+60d:  user викликає unstake(400, nonce=1234)
        → settle (як в harvest)
        → user.amount_staked = 600
        → pool.total_staked -= 400
        → створюється UnstakeRequest{ amount: 400, created_at: T+60d }

T+67d:  user викликає claim(unstake_request)
        → require!(now >= created_at + 7d)  ✓
        → CPI transfer 400 з vault → user
        → close UnstakeRequest, rent → user
```

---

## Що це демонструє у портфоліо

- **Anchor-фреймворк full-stack**: PDA, CPI, signer_seeds, has_one, init/init_if_needed/close, events.
- **Рівень DeFi-математики**: Synthetix accumulator pattern — стандарт для всіх V2 staking-протоколів.
- **Безпекове мислення**: розв'язка `request_time` vs `created_at`, settlement-before-mutation, re-entrancy hygiene.
- **Інженерна гігієна**: `checked_*` арифметика, явні error codes, події для індексаторів, повне інтеграційне покриття через LiteSVM.
