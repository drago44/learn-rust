# Чим це відрізняється від реальних протоколів

> Контекст: цей протокол — навчальний (див. [README.md](./README.md), [HOW_IT_WORKS.md](./HOW_IT_WORKS.md)). Тут пояснено, **які компоненти реальних staking-протоколів у нас є, а які ні**, і як великі протоколи реально заробляють і рахують APY.

На рівні контракту — **це та сама система**. Synthetix-style accumulator, що ми написали, є інфраструктурним шаром у переважній більшості великих staking/farming протоколів (Synthetix, SushiSwap MasterChef, Convex, GMX, Marinade, Jito). Різниця **не в коді розподілу**, а в тому, **звідки беруться reward-токени** і **як рахується APY поверх цього**.

---

## Звідки реальні протоколи беруть reward-budget

У нашій моделі адмін просто кладе токени в `reward_vault` — звідки вони беруться, програма не питає. Реальні протоколи мають **економічне джерело**, яке генерує reward-потік.

### 1. Trading fees (DEXes: Orca, Raydium, Jupiter)

Користувач робить swap → платить ~0.3% fee → **частина** йде LP-власникам, **частина** — окремим стейкерам governance-токена (наприклад, `xORCA`).

```
Trade $1000 → fee $3 → $2.5 LP, $0.5 stakers
```

`reward_rate` тут не константа — він **росте з обсягом торгівлі**. Часто протокол просто акумулює fee у `reward_vault` і розподіляє через ту саму accumulator-формулу що в нас.

### 2. Borrow interest (Lending: Solend, Kamino, MarginFi)

Юзер позичає під ~8% APR → платить interest → це **revenue протоколу** → частина йде стейкерам, частина у treasury.

Pattern: `total_borrowed × interest_rate × time` = revenue. Розподіляється за тим самим accumulator.

### 3. Validator rewards + MEV (Liquid staking: Marinade, Jito, Lido)

Юзер вносить SOL → отримує `mSOL`/`jitoSOL` → SOL делегується валідаторам → **валідатори заробляють** ~7% APR з inflation + MEV → reward повертається назад у пул, ціна `mSOL` росте відносно SOL.

Це не зовсім наш паттерн — там **token price appreciation** замість окремих rewards. Але економічна суть та сама — revenue від реальної on-chain активності.

### 4. Token emissions (Curve, Convex, більшість farming-протоколів)

Як у нас. Treasury володіє великим запасом токена → виплачує його стейкерам як emissions. Це **не sustainable revenue** — це **subsidiary**: протокол роздає supply, щоб залучити TVL і користувачів. Працює рік-два, поки не кінчиться runway.

```
Treasury (1B токенів) ──emit_per_second──▶ stakers
```

Цю модель **наш код підтримує без змін** — просто треба, щоб адмін періодично робив `mint_to(reward_vault, X)` зі своєї `mint_authority`.

### 5. Real-world yield (Maple, Ondo, Centrifuge)

Протокол позичає реальним компаніям під 12% або тримає US Treasuries → on-chain отримує yield → **транслює** його стейкерам через таку саму accumulator-механіку.

---

## Як рахується APY

APY — це **off-chain метрика**, не on-chain величина. Формула проста:

```
APR = (reward_rate × seconds_per_year × price_reward_token)
       ÷ (total_staked × price_stake_token)

APY = (1 + APR/n)^n − 1      ← з врахуванням compounding
```

де `n` — частота капіталізації. Для безперервного нарахування: `APY ≈ e^APR − 1`.

### Що береться on-chain

- `reward_rate` — з `pool` акаунта (у нас теж є);
- `total_staked` — звідти ж;
- ціни — **поза контрактом** (Pyth/Switchboard оракули або CoinGecko через бекенд).

### Хто рахує

- **Frontend** (dApp) — fetch'ить ці три числа і малює `APY: 12.4%`.
- **Aggregator** (DefiLlama, Step Finance) — те саме, але глобально.

Програма **не зобов'язана знати APY** — це деривативна метрика. Можна додати `view_apr()` як off-chain helper, але on-chain — зайве (марно жгти compute).

### Чому APY постійно змінюється

```
APR = (reward_rate × seconds_per_year × $reward) / (total_staked × $stake)
                  ▲                                       ▲
              rate змінює admin                     росте з притоком
```

Простий приклад:

- стартує пул, TVL = $10k → APR ≈ 100% (всі rewards діляться між кількома);
- TVL росте до $1M → APR падає до ≈1% (ті ж rewards діляться між тисячами);
- адмін підвищує `reward_rate` у 10× → APR назад до ≈10%.

Це **і є** «крутіння APY» — балансування між залученням капіталу та витратами на емісію.

---

## Що в нашому контракті є, а що ні

| Компонент                              | У нашому коді | У реальному протоколі      |
|----------------------------------------|---------------|----------------------------|
| Accumulator-розподіл rewards           | ✅            | ✅ (часто слово-в-слово)   |
| Cooldown + claim                       | ✅            | різниться (0–28 днів)      |
| Settlement-before-mutation             | ✅            | ✅                         |
| Адмін `update_reward_rate`             | ✅            | ✅ (але через DAO/timelock)|
| `fund_rewards` зі сторонніх джерел     | ❌            | завжди є                   |
| Hard cap на `reward_rate`              | ❌            | часто є (anti-rug)         |
| Timelock на admin actions              | ❌            | mainnet-стандарт           |
| Multi-sig authority                    | ❌            | mainnet-стандарт           |
| APY view                               | ❌ (off-chain)| off-chain                  |
| Інтеграція з оракулами цін             | ❌            | для APY-рендеру            |
| Auto-compound                          | ❌            | у деяких є (Yearn-style)   |
| Slashing / jail                        | ❌            | у liquid staking є         |

Наш контракт — це **80% продакшн staking-протоколу**. Інші 20% — це **governance-обв'язка** (timelock, multisig, hard caps) і **економічний layer** (звідки беруться rewards). Це ортогональні питання, не ламають архітектуру.

---

## Коротка відповідь

Так, **та сама система**. Те що ми написали — це **мотор розподілу**, який однаковий у 95% staking-протоколів. Реальні протоколи додають зверху:

1. **Економічне джерело** rewards (fees / interest / emissions).
2. **Off-chain APY калькулятор** з оракулами цін.
3. **Governance-обв'язку** (timelock, multisig, hard caps).

Сама математика розподілу — наш код. Це найскладніша і найкритичніша частина, і вона у нас зроблена правильно.
