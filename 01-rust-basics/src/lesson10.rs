// ============================================================
// Урок 10 — Testing
// ============================================================
//
// Rust має вбудовану систему тестів — не потрібні зовнішні бібліотеки.
// `cargo test` знаходить всі функції з #[test] і запускає їх.

pub fn run() {
    println!("=== Testing у Rust ===");
    println!("  Запусти: cargo test");
    println!("  З виводом: cargo test -- --nocapture");
    println!("  Один тест: cargo test test_add");
    println!("  За модулем: cargo test unit_tests");
    println!();
    println!("  Цей урок — про тести, не про run().");
    println!("  Весь корисний код — в #[cfg(test)] блоці нижче.");
    println!();

    // Функції що тестуємо — публічні, доступні і в run() і в тестах
    println!("  Демо функцій що тестуються:");
    println!("  add(2, 3) = {}", add(2, 3));
    println!("  divide(10, 2) = {:?}", divide(10.0, 2.0));
    println!("  divide(10, 0) = {:?}", divide(10.0, 0.0));
    println!("  is_palindrome(\"racecar\") = {}", is_palindrome("racecar"));
    println!("  is_palindrome(\"hello\") = {}", is_palindrome("hello"));
    println!("  fizzbuzz(15) = {}", fizzbuzz(15));
}

// ============================================================
// ФУНКЦІЇ ЩО ТЕСТУЄМО
// ============================================================

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[allow(dead_code)]
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("division by zero"))
    } else {
        Ok(a / b)
    }
}

pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let reversed: Vec<char> = chars.iter().rev().cloned().collect();
    chars == reversed
}

pub fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => String::from("FizzBuzz"),
        (0, _) => String::from("Fizz"),
        (_, 0) => String::from("Buzz"),
        _ => n.to_string(),
    }
}

// Функція що панікує — для тестування #[should_panic]
#[allow(dead_code)]
pub fn must_be_positive(n: i32) -> i32 {
    if n <= 0 {
        panic!("value must be positive, got {n}");
    }
    n
}

// ============================================================
// ТЕСТИ
// ============================================================
//
// #[cfg(test)] — цей блок компілюється тільки при `cargo test`.
// Не потрапляє в release бінарник.
//
// Структура тесту:
//   #[test]
//   fn test_something() {
//       // arrange
//       // act
//       // assert
//   }

#[cfg(test)]
mod unit_tests {
    // `use super::*` — імпортуємо все з батьківського модуля
    use super::*;

    // --------------------------------------------------------
    // assert! — перевіряє що вираз = true
    // assert_eq! — перевіряє рівність (показує обидва значення при провалі)
    // assert_ne! — перевіряє нерівність
    // --------------------------------------------------------

    #[test]
    fn test_add_basic() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, -1), -2);
        assert_eq!(add(-5, 3), -2);
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(42, 0), 42);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(10, 3), 7);
        assert_ne!(subtract(10, 3), 8); // assert_ne: НЕ дорівнює
    }

    // --------------------------------------------------------
    // Тест з власним повідомленням при провалі
    // assert_eq!(left, right, "message {}", var)
    // --------------------------------------------------------

    #[test]
    fn test_fizzbuzz() {
        assert_eq!(fizzbuzz(1), "1", "1 має бути просто числом");
        assert_eq!(fizzbuzz(3), "Fizz", "3 ділиться на 3");
        assert_eq!(fizzbuzz(5), "Buzz", "5 ділиться на 5");
        assert_eq!(fizzbuzz(15), "FizzBuzz", "15 ділиться на 3 і 5");
        assert_eq!(fizzbuzz(30), "FizzBuzz");
    }

    #[test]
    fn test_fizzbuzz_range() {
        let expected = ["1", "2", "Fizz", "4", "Buzz", "Fizz", "7", "8", "Fizz", "Buzz"];
        for (i, &exp) in expected.iter().enumerate() {
            let n = (i + 1) as u32;
            assert_eq!(fizzbuzz(n), exp, "fizzbuzz({n}) failed");
        }
    }

    // --------------------------------------------------------
    // Тести на Result — два підходи
    // --------------------------------------------------------

    // Підхід 1: unwrap + expect
    #[test]
    fn test_divide_ok() {
        let result = divide(10.0, 2.0).expect("division should succeed");
        assert_eq!(result, 5.0);
    }

    // Підхід 2: assert!(result.is_ok()) / assert!(result.is_err())
    #[test]
    fn test_divide_by_zero() {
        let result = divide(10.0, 0.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "division by zero");
    }

    // Підхід 3: Result<(), E> як тип повернення тесту
    // При Err — тест падає з повідомленням помилки
    #[test]
    fn test_divide_result_return() -> Result<(), String> {
        let result = divide(9.0, 3.0)?; // ? прокидає Err → тест падає
        assert_eq!(result, 3.0);
        Ok(())
    }

    // --------------------------------------------------------
    // #[should_panic] — тест що очікує паніку
    // --------------------------------------------------------

    #[test]
    #[should_panic]
    fn test_panic_without_message() {
        must_be_positive(-1); // має запанікувати
    }

    // Точніше: перевіряємо текст паніки через expected
    #[test]
    #[should_panic(expected = "value must be positive")]
    fn test_panic_with_message() {
        must_be_positive(0);
    }

    // --------------------------------------------------------
    // Тести на bool-функції
    // --------------------------------------------------------

    #[test]
    fn test_palindrome_true() {
        assert!(is_palindrome("racecar"));
        assert!(is_palindrome("madam"));
        assert!(is_palindrome("a"));
        assert!(is_palindrome(""));
    }

    #[test]
    fn test_palindrome_false() {
        assert!(!is_palindrome("hello"));
        assert!(!is_palindrome("rust"));
    }

    // --------------------------------------------------------
    // #[ignore] — пропустити тест (але він є в списку)
    // Запустити: cargo test -- --ignored
    // --------------------------------------------------------

    #[test]
    #[ignore = "slow test, run manually"]
    fn test_slow_operation() {
        // Імітація повільного тесту
        let sum: u64 = (1..=1_000_000).sum();
        assert_eq!(sum, 500_000_500_000);
    }
}

// ============================================================
// ТЕСТИ З HELPER ФУНКЦІЯМИ
// ============================================================
//
// Великі тест-модулі часто мають helper функції що не є тестами.
// Без #[test] — просто звичайні функції всередині #[cfg(test)].

#[cfg(test)]
mod helpers_demo {
    use super::*;

    fn make_test_cases() -> Vec<(u32, &'static str)> {
        vec![
            (1, "1"),
            (3, "Fizz"),
            (5, "Buzz"),
            (15, "FizzBuzz"),
        ]
    }

    #[test]
    fn test_fizzbuzz_with_helper() {
        for (input, expected) in make_test_cases() {
            assert_eq!(
                fizzbuzz(input),
                expected,
                "fizzbuzz({input}) should be {expected}"
            );
        }
    }

    #[test]
    fn test_add_commutative() {
        // Властивість: add(a, b) == add(b, a)
        let pairs = [(1, 2), (0, 5), (-3, 7), (100, -100)];
        for (a, b) in pairs {
            assert_eq!(
                add(a, b),
                add(b, a),
                "add not commutative for ({a}, {b})"
            );
        }
    }
}

// ============================================================
// INTEGRATION-STYLE ТЕСТИ (в межах модуля)
// ============================================================
//
// Справжні integration тести — в окремій папці tests/
// Але можна імітувати в одному файлі через окремий mod.
// Integration тести перевіряють що функції РАЗОМ дають правильний результат.

#[cfg(test)]
mod integration_style {
    use super::*;

    #[test]
    fn test_math_pipeline() {
        // add, потім divide — разом
        let sum = add(6, 4) as f64;
        let result = divide(sum, 2.0).expect("no division by zero");
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_fizzbuzz_all_categories_present() {
        // Перевіряємо що у 1..=15 є всі чотири категорії
        let results: Vec<String> = (1..=15).map(fizzbuzz).collect();
        assert!(results.iter().any(|r| r == "Fizz"));
        assert!(results.iter().any(|r| r == "Buzz"));
        assert!(results.iter().any(|r| r == "FizzBuzz"));
        assert!(results.iter().any(|r| r.parse::<u32>().is_ok()));
    }
}
