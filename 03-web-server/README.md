# Місяць 3 ✅ — Web сервер

**Що вивчаємо:**
##### ✅ `axum` — routing, handlers, middleware, extractors
##### ✅ `tower` — rate limiting на auth endpoints через `tower_governor`
##### ✅ `SeaORM` + SQLite — async ORM, міграції через `MigratorTrait`
##### ✅ JWT аутентифікація — access token (15хв) + refresh token (7 днів)
##### ✅ REST API патерни

**Проєкт: `03-web-server/`**
##### ✅ `GET /health` — статус сервера
##### ✅ `GET /coins` — список монет з CoinGecko
##### ✅ `GET /prices/{symbol}` — поточна ціна
##### ✅ `POST /auth/register` — реєстрація (argon2 хешування)
##### ✅ `POST /auth/login` — логін, повертає access + refresh токени
##### ✅ `POST /auth/refresh` — оновлення access токена
##### ✅ `POST /auth/logout` — відкликання refresh токена (захищений JWT)
##### ✅ `POST /portfolio` — створити портфель (з JWT)
##### ✅ `GET /portfolio` — отримати портфель
##### ✅ `POST /portfolio/asset` — додати актив
##### ✅ `DELETE /portfolio/asset/{symbol}` — видалити
##### ✅ SQLite для зберігання (через SeaORM)
##### ✅ JWT middleware — перевірка Bearer токена на захищених маршрутах
##### ✅ Rate limiting на `/auth/login` та `/auth/register` (`tower_governor`)
##### ✅ Тести для всіх endpoint'ів

**Архітектура:**
```
src/
  dto/          ← request/response типи (не йдуть в DB)
  handlers/     ← тонкі хендлери, тільки routing
  services/     ← бізнес-логіка (auth, coingecko)
  repositories/ ← SeaORM запити до DB
  models/       ← SeaORM entities (відображення таблиць)
  migration/    ← міграції через MigratorTrait з up()/down()
  middleware/   ← JWT перевірка
```
