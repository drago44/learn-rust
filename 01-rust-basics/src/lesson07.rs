// ============================================================
// Урок 7 — Modules та Crates
// ============================================================
//
// Модулі — система організації коду в Rust.
// За замовчуванням все ПРИВАТНЕ. pub робить публічним.
// Модулі створюють ієрархію: crate::module::submodule::item
//
// Файлова структура:
//   mod foo;         → шукає foo.rs або foo/mod.rs
//   mod foo { ... }  → inline-модуль прямо у файлі
//
// Видимість:
//   (нічого)   — приватне (тільки поточний модуль)
//   pub        — повністю публічне
//   pub(crate) — видиме в межах crate, але не ззовні
//   pub(super) — видиме в батьківському модулі

pub fn run() {
    println!("=== mod та приватність ===");
    lesson_mod_basics();

    println!("\n=== use — імпорт ===");
    lesson_use_imports();

    println!("\n=== pub(crate) та pub(super) — обмежена видимість ===");
    lesson_restricted_visibility();

    println!("\n=== Структури в модулях — видимість полів ===");
    lesson_struct_visibility();

    println!("\n=== self, super, crate — навігація по дереву модулів ===");
    lesson_path_navigation();

    println!("\n=== Зовнішні crates та Cargo.toml ===");
    lesson_external_crates();

    println!("\n=== Re-export з pub use ===");
    lesson_reexport();
}

// ============================================================
// MOD BASICS — inline-модулі та приватність
// ============================================================

mod network {
    // Приватна — видима тільки всередині mod network
    fn internal_check() -> bool {
        true
    }

    // pub — доступна ззовні модуля
    pub fn connect(addr: &str) -> String {
        if internal_check() {
            format!("Connected to {addr}")
        } else {
            format!("Failed to connect to {addr}")
        }
    }

    pub fn disconnect() -> String {
        String::from("Disconnected")
    }

    // Вкладений модуль
    pub mod dns {
        pub fn resolve(host: &str) -> String {
            format!("192.168.1.1 (resolved {host})")
        }

        // Приватна — не видима за межами dns
        #[allow(dead_code)]
        fn cache_lookup(_host: &str) -> Option<String> {
            None
        }
    }
}

fn lesson_mod_basics() {
    // Доступ через повний шлях: модуль::функція
    let result = network::connect("localhost:8080");
    println!("  {result}");

    // Вкладений модуль — подвійний шлях
    let ip = network::dns::resolve("example.com");
    println!("  DNS: {ip}");

    // network::internal_check();      // ПОМИЛКА! Приватна функція
    // network::dns::cache_lookup("x"); // ПОМИЛКА! Приватна
}

// ============================================================
// USE — імпорт для скорочення шляхів
// ============================================================
//
// use дозволяє "імпортувати" елемент, щоб не писати повний шлях.
// Кілька варіантів:
//   use module::function;           — конкретний елемент
//   use module::{a, b, c};          — кілька елементів
//   use module::*;                  — все публічне (glob, уникай)

// Імпортуємо конкретні елементи
use network::connect;
use network::dns::resolve;

fn lesson_use_imports() {
    // Тепер можна без повного шляху
    let result = connect("10.0.0.1:443");
    println!("  {result}");

    let ip = resolve("rust-lang.org");
    println!("  DNS: {ip}");

    // Повний шлях все ще працює
    let disc = network::disconnect();
    println!("  {disc}");

    // Демо вкладеного use
    nested_use_demo();
}

fn nested_use_demo() {
    // Можна імпортувати кілька речей з одного шляху
    // use network::{connect, disconnect};
    // use std::collections::{HashMap, HashSet, BTreeMap};
    //
    // self в use — сам модуль + його елементи:
    // use std::io::{self, Read, Write};
    // еквівалент:
    // use std::io;
    // use std::io::Read;
    // use std::io::Write;

    println!("  (nested use: use module::{{a, b, c}})");
}

// ============================================================
// PUB(CRATE) та PUB(SUPER) — обмежена видимість
// ============================================================

mod auth {
    // pub(crate) — видиме всюди в цьому crate, але не ззовні
    // Ідеально для внутрішніх API бібліотеки
    pub(crate) fn validate_token(token: &str) -> bool {
        !token.is_empty() && token.len() > 8
    }

    // pub(super) — видиме тільки в батьківському модулі
    pub(super) fn get_secret() -> &'static str {
        "super-secret-key-123"
    }

    // Звичайна pub — видима всім
    pub fn login(user: &str) -> String {
        if validate_token(user) {
            format!("Welcome, {user}")
        } else {
            format!("Invalid credentials for {user}")
        }
    }

    pub mod roles {
        // pub(super) — видиме тільки в mod auth (батько roles)
        #[allow(dead_code)]
        pub(super) fn is_admin(user: &str) -> bool {
            user == "admin"
        }

        // pub(crate) — видиме в усьому crate
        pub(crate) fn default_role() -> &'static str {
            "viewer"
        }
    }
}

fn lesson_restricted_visibility() {
    // pub — доступна
    println!("  {}", auth::login("admin-token-12345"));

    // pub(crate) — доступна бо ми в тому ж crate
    let valid = auth::validate_token("my-long-token");
    println!("  Token valid: {valid}");

    // pub(super) — доступна бо lesson07 є батьком auth
    let secret = auth::get_secret();
    println!("  Secret: {secret}");

    // pub(crate) з вкладеного модуля
    let role = auth::roles::default_role();
    println!("  Default role: {role}");

    // auth::roles::is_admin("x"); // Працює тільки з mod auth, не звідси
}

// ============================================================
// СТРУКТУРИ В МОДУЛЯХ — видимість полів
// ============================================================
//
// pub struct робить структуру видимою, але поля залишаються приватними!
// Кожне поле потрібно окремо позначити pub.

mod config {
    #[derive(Debug)]
    pub struct DatabaseConfig {
        pub host: String,       // публічне — можна читати/писати ззовні
        pub port: u16,          // публічне
        password: String,       // ПРИВАТНЕ — тільки всередині mod config
    }

    impl DatabaseConfig {
        // Конструктор — єдиний спосіб створити структуру з приватними полями
        pub fn new(host: &str, port: u16, password: &str) -> Self {
            Self {
                host: host.to_string(),
                port,
                password: password.to_string(),
            }
        }

        // Геттер для приватного поля
        pub fn has_password(&self) -> bool {
            !self.password.is_empty()
        }

        // Маскований вивід
        pub fn masked_password(&self) -> String {
            if self.password.is_empty() {
                String::from("(none)")
            } else {
                format!("{}***", &self.password[..2.min(self.password.len())])
            }
        }
    }

    // Enum — на відміну від struct, варіанти pub enum ВСІ публічні
    #[derive(Debug)]
    #[allow(dead_code)]
    pub enum Environment {
        Development,
        Staging,
        Production,
    }
}

fn lesson_struct_visibility() {
    // Не можна створити напряму — password приватне:
    // let cfg = config::DatabaseConfig { host: ..., port: ..., password: ... }; // ПОМИЛКА!

    // Тільки через конструктор
    let cfg = config::DatabaseConfig::new("localhost", 5432, "s3cret");
    println!("  Config: {}:{}", cfg.host, cfg.port); // pub поля — ок
    // println!("{}", cfg.password); // ПОМИЛКА! Приватне поле
    println!("  Has password: {}", cfg.has_password());
    println!("  Password: {}", cfg.masked_password());

    // Enum варіанти — всі публічні
    let env = config::Environment::Production;
    println!("  Env: {env:?}");
}

// ============================================================
// SELF, SUPER, CRATE — навігація по дереву модулів
// ============================================================
//
// crate:: — абсолютний шлях від кореня crate
// super:: — батьківський модуль (як .. в файловій системі)
// self::  — поточний модуль (як . в файловій системі)

mod services {
    // Функція на рівні services
    pub fn base_url() -> &'static str {
        "https://api.example.com"
    }

    pub mod api {
        // super:: — доступ до батьківського модуля (services)
        pub fn endpoint() -> String {
            let base = super::base_url(); // services::base_url()
            format!("{base}/v1/data")
        }

        pub mod v2 {
            // super::super:: — два рівні вгору
            pub fn endpoint() -> String {
                let base = super::super::base_url(); // services::base_url()
                format!("{base}/v2/data")
            }
        }
    }

    pub mod internal {
        // Можна використовувати self:: для явності (необов'язково)
        fn helper() -> &'static str {
            "internal helper"
        }

        pub fn do_work() -> String {
            let h = self::helper(); // те саме що helper()
            format!("Working with {h}")
        }
    }
}

fn lesson_path_navigation() {
    println!("  Base: {}", services::base_url());
    println!("  API v1: {}", services::api::endpoint());
    println!("  API v2: {}", services::api::v2::endpoint());
    println!("  Internal: {}", services::internal::do_work());

    // crate:: — абсолютний шлях від кореня
    // В main.rs: crate::lesson07::run()
    // В lesson07.rs: crate::lesson07::network::connect(...)
    println!("  (crate:: = абсолютний шлях від кореня crate)");
}

// ============================================================
// ЗОВНІШНІ CRATES — Cargo.toml
// ============================================================
//
// Rust використовує Cargo для управління залежностями.
//
// Додати crate:
//   cargo add serde           → додає в Cargo.toml
//   cargo add serde --features derive
//
// Cargo.toml:
//   [dependencies]
//   serde = "1.0"
//   serde_json = "1.0"
//   reqwest = { version = "0.12", features = ["json"] }
//
// Потім у коді:
//   use serde::{Serialize, Deserialize};
//   use reqwest;
//
// crates.io — реєстр пакетів Rust:
//   https://crates.io/
//   Шукай за категорією: serialization, http, cli, crypto...
//
// Популярні crates:
//   serde + serde_json — серіалізація (JSON, TOML, YAML...)
//   tokio              — async runtime
//   reqwest            — HTTP клієнт
//   clap               — CLI аргументи
//   anyhow / thiserror — error handling
//   tracing            — логування
//   rand               — рандом

fn lesson_external_crates() {
    println!("  Додати crate: cargo add <name>");
    println!("  Пошук: https://crates.io/");
    println!("  Приклад Cargo.toml:");
    println!("    [dependencies]");
    println!("    serde = {{ version = \"1.0\", features = [\"derive\"] }}");
    println!("    tokio = {{ version = \"1\", features = [\"full\"] }}");

    // Стандартна бібліотека (std) доступна без Cargo.toml:
    use std::collections::HashMap;
    let mut m = HashMap::new();
    m.insert("std", "завжди доступна");
    println!("  std::collections::HashMap: {:?}", m);
}

// ============================================================
// PUB USE — re-export
// ============================================================
//
// pub use дозволяє "підняти" елемент з глибини модуля
// на рівень вище. Користувачі бібліотеки бачать простий API,
// а внутрішня структура може бути складною.

mod engine {
    mod core {
        pub fn process(data: &str) -> String {
            format!("processed: {data}")
        }
    }

    mod utils {
        pub fn validate(data: &str) -> bool {
            !data.is_empty()
        }
    }

    // Re-export — піднімаємо на рівень engine::
    // Замість engine::core::process → engine::process
    pub use core::process;
    pub use utils::validate;
}

fn lesson_reexport() {
    // Завдяки pub use — простий шлях
    let result = engine::process("hello");
    println!("  {result}");

    let valid = engine::validate("test");
    println!("  Valid: {valid}");

    // Без pub use довелося б писати:
    // engine::core::process("hello")    — і core має бути pub
    // engine::utils::validate("test")   — і utils має бути pub
    //
    // pub use — ідіоматичний спосіб створити чистий публічний API
    println!("  (pub use = чистий API без розкриття внутрішньої структури)");
}
