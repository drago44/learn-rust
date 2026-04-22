# Місяць 2 ✅ — Async Rust + перший реальний проєкт

**Що вивчаємо:**
##### ✅ `async`/`await` — синтаксис, Future trait
##### ✅ `tokio` — runtime, `tokio::spawn`, `tokio::join!`
##### ✅ `reqwest` — HTTP запити (async)
##### ✅ `serde` / `serde_json` — серіалізація/десеріалізація
##### ✅ `std::fs` / `tokio::fs` — робота з файлами
##### ✅ `clap` — CLI аргументи та subcommands
##### ✅ `thiserror` / `anyhow` — production error handling

**Проєкт: `02-portfolio-tracker/`**
##### ✅ `cargo new portfolio-tracker`
##### ✅ Команда `add <symbol> <amount>` — додати монету в портфель
##### ✅ Команда `remove <symbol>` — видалити
##### ✅ Команда `list` — показати портфель з поточними цінами
##### ✅ Команда `total` — загальна вартість у USD
##### ✅ Зберігання портфелю в JSON файлі (`~/.portfolio.json`)
##### ✅ Ціни з публічного API (CoinGecko або CoinCap)
##### ✅ Async оновлення кількох цін паралельно (`tokio::join!`)
##### ✅ Повноцінний error handling через `anyhow`
