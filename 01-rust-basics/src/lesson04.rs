// ============================================================
// Урок 4 — Error Handling (Result<T, E>)
// ============================================================
//
// Result<T, E> — enum для операцій, що можуть провалитись:
//   Ok(T)   — успіх, містить значення
//   Err(E)  — помилка, містить причину
//
// Порівняння з Option<T>:
//   Option<T>    → Some(value) | None         — "є чи нема"
//   Result<T, E> → Ok(value)  | Err(error)   — "вдалось чи помилка"

pub fn run() {
    println!("=== Result<T, E> основи ===");
    lesson_result_basics();

    println!("\n=== Оператор ? ===");
    lesson_question_mark();

    println!("\n=== Власний enum помилок ===");
    lesson_custom_error();

    println!("\n=== unwrap, expect, конвертації ===");
    lesson_unwrap_and_conversions();
}

// ============================================================
// RESULT BASICS
// ============================================================

// Парсимо рядок у баланс. Може провалитись з двох причин:
// 1. Рядок не є числом → помилка парсингу
// 2. Число від'ємне → бізнес-логіка забороняє
fn parse_balance(input: &str) -> Result<f64, String> {
    // .parse::<f64>() повертає Result<f64, ParseFloatError>
    // .map_err() конвертує тип помилки: ParseFloatError → String
    let value = input.parse::<f64>().map_err(|e| e.to_string())?;

    if value < 0.0 {
        return Err("balance cannot be negative".to_string());
    }

    Ok(value)
}

fn lesson_result_basics() {
    let inputs = ["42.5", "abc", "-10"];

    for input in inputs {
        match parse_balance(input) {
            Ok(balance) => println!("  \"{input}\" → Ok({balance:.2})"),
            Err(e) => println!("  \"{input}\" → Err({e})"),
        }
    }
}

// ============================================================
// ОПЕРАТОР ?
// ============================================================
//
// ? — синтаксичний цукор. Замість:
//   let value = match something() {
//       Ok(v) => v,
//       Err(e) => return Err(e),
//   };
// Пишемо:
//   let value = something()?;
//
// Якщо Ok — розгортає значення.
// Якщо Err — одразу return Err з функції.
// Працює ТІЛЬКИ у функціях, що повертають Result.

fn create_wallet(name: &str, balance_str: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("name cannot be empty".to_string());
    }

    // ? прокидає помилку з parse_balance вгору
    let balance = parse_balance(balance_str)?;

    Ok(format!("Wallet: {} ({:.2})", name, balance))
}

fn lesson_question_mark() {
    let cases = [
        ("Alice", "100.0"),
        ("", "50.0"),       // порожнє ім'я
        ("Bob", "xyz"),     // невалідне число
        ("Carol", "-5"),    // від'ємне
    ];

    for (name, balance) in cases {
        match create_wallet(name, balance) {
            Ok(wallet) => println!("  Ok: {wallet}"),
            Err(e) => println!("  Err(\"{name}\", \"{balance}\"): {e}"),
        }
    }
}

// ============================================================
// ВЛАСНИЙ ТИП ПОМИЛКИ
// ============================================================
//
// В реальних проєктах (і в Solana/Anchor) кожна програма
// має свій enum помилок. Це дає:
// - Чіткі типи замість String
// - Вичерпний match — компілятор перевірить всі варіанти
// - Легке додавання нових помилок

#[derive(Debug)]
enum WalletError {
    InvalidAmount(String), // рядок не парситься в число
    NegativeBalance,       // від'ємна сума
    EmptyName,             // порожнє ім'я
}

fn parse_balance_typed(input: &str) -> Result<f64, WalletError> {
    let value = input
        .parse::<f64>()
        .map_err(|e| WalletError::InvalidAmount(e.to_string()))?;

    if value < 0.0 {
        return Err(WalletError::NegativeBalance);
    }

    Ok(value)
}

fn create_wallet_typed(name: &str, balance_str: &str) -> Result<String, WalletError> {
    if name.is_empty() {
        return Err(WalletError::EmptyName);
    }

    let balance = parse_balance_typed(balance_str)?;

    Ok(format!("Wallet: {} ({:.2})", name, balance))
}

fn lesson_custom_error() {
    let cases = [
        ("Alice", "100.0"),
        ("", "50.0"),
        ("Bob", "xyz"),
        ("Carol", "-5"),
    ];

    for (name, balance) in cases {
        match create_wallet_typed(name, balance) {
            Ok(wallet) => println!("  Ok: {wallet}"),
            Err(WalletError::EmptyName) => {
                println!("  EmptyName: ім'я не може бути порожнім")
            }
            Err(WalletError::InvalidAmount(msg)) => {
                println!("  InvalidAmount: \"{balance}\" — {msg}")
            }
            Err(WalletError::NegativeBalance) => {
                println!("  NegativeBalance: баланс не може бути від'ємним")
            }
        }
    }
}

// ============================================================
// UNWRAP, EXPECT, КОНВЕРТАЦІЇ
// ============================================================

fn lesson_unwrap_and_conversions() {
    // --- unwrap / expect ---
    // unwrap() — витягує Ok або ПАНІКА якщо Err.
    // Використовуй тільки коли 100% впевнений або в тестах.
    let sure: Result<i32, &str> = Ok(42);
    println!("  unwrap: {}", sure.unwrap());

    // expect() — як unwrap, але з повідомленням при паніці.
    // Краще за unwrap — бачиш ЩО пішло не так.
    let also_sure: Result<i32, &str> = Ok(7);
    println!("  expect: {}", also_sure.expect("це не повинно впасти"));

    // --- unwrap_or / unwrap_or_else ---
    // Безпечні альтернативи — fallback замість паніки.
    let bad: Result<i32, &str> = Err("щось пішло не так");
    println!("  unwrap_or: {}", bad.unwrap_or(0)); // 0

    let bad2: Result<i32, &str> = Err("помилка");
    let val = bad2.unwrap_or_else(|e| {
        println!("  unwrap_or_else отримав помилку: {e}");
        -1
    });
    println!("  fallback значення: {val}");

    // --- Result → Option ---
    // .ok() відкидає помилку, залишає тільки значення
    let good: Result<i32, &str> = Ok(10);
    let as_option: Option<i32> = good.ok();
    println!("  Result→Option (Ok): {:?}", as_option); // Some(10)

    let bad3: Result<i32, &str> = Err("oops");
    let as_option2: Option<i32> = bad3.ok();
    println!("  Result→Option (Err): {:?}", as_option2); // None

    // --- Option → Result ---
    // .ok_or() додає помилку до None
    let some: Option<i32> = Some(5);
    let as_result: Result<i32, &str> = some.ok_or("значення відсутнє");
    println!("  Option→Result (Some): {:?}", as_result); // Ok(5)

    let none: Option<i32> = None;
    let as_result2: Result<i32, &str> = none.ok_or("значення відсутнє");
    println!("  Option→Result (None): {:?}", as_result2); // Err("значення відсутнє")
}
