// ============================================================
// Урок 3 — Structs та Enums
// ============================================================
//
// Struct — іменована група полів (як об'єкт/record).
// Enum — тип із кількома варіантами (як tagged union).
// Разом вони — основа моделювання даних у Rust.

pub fn run() {
    println!("=== Struct ===");
    lesson_struct();

    println!("\n=== Tuple struct ===");
    lesson_tuple_struct();

    println!("\n=== Enum ===");
    lesson_enum();

    println!("\n=== Enum з даними ===");
    lesson_enum_with_data();

    println!("\n=== Option<T> ===");
    lesson_option();

    println!("\n=== if let ===");
    lesson_if_let();
}

// ============================================================
// STRUCTS
// ============================================================

// --- Struct: іменовані поля ---
// Кожне поле має ім'я та тип. Всі поля обов'язкові при створенні.
struct Wallet {
    owner: String,
    balance: f64,
    active: bool,
}

// --- impl блок: тут живуть методи ---
// Self — синонім назви struct (тут = Wallet).
// Можна мати кілька impl блоків для одного struct.
impl Wallet {
    // Associated function (немає self) — викликається Wallet::new(...)
    // Це конвенція для конструкторів у Rust.
    fn new(owner: &str, balance: f64) -> Self {
        Self {
            owner: String::from(owner),
            balance, // shorthand: якщо поле і змінна мають однакове ім'я
            active: true,
        }
    }

    // &self — immutable borrow, метод тільки читає
    fn display(&self) {
        let status = if self.active { "active" } else { "inactive" };
        println!("  {} : {:.2} ({})", self.owner, self.balance, status);
    }

    // &mut self — mutable borrow, метод змінює дані
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    // Повертає bool — чи вдалося зняти
    fn withdraw(&mut self, amount: f64) -> bool {
        if amount > self.balance {
            println!("  недостатньо коштів!");
            return false;
        }
        self.balance -= amount;
        true
    }

    fn deactivate(&mut self) {
        self.active = false;
    }
}

fn lesson_struct() {
    // Створення через конструктор
    let mut wallet = Wallet::new("Alice", 100.0);
    wallet.display();

    // Депозит
    wallet.deposit(50.0);
    println!("після депозиту:");
    wallet.display();

    // Виведення — успішне
    let ok = wallet.withdraw(30.0);
    println!("withdraw 30: {ok}");
    wallet.display();

    // Виведення — недостатньо коштів
    let fail = wallet.withdraw(999.0);
    println!("withdraw 999: {fail}");

    // Деактивація
    wallet.deactivate();
    wallet.display();

    // --- Struct update syntax ---
    // Створюємо новий struct на основі існуючого, замінюючи частину полів.
    // ..wallet — "решту полів візьми з wallet".
    // Увага: це MOVE для String полів (owner переміщується).
    let wallet2 = Wallet {
        owner: String::from("Bob"), // нове значення
        ..wallet                    // balance та active з wallet
    };
    wallet2.display();
    // wallet.owner тепер недоступний (move), але wallet.balance — Copy, ще живий
}

// --- Tuple struct: struct без імен полів ---
// Корисно для "newtype pattern" — обгортка навколо типу для type safety.
// Celsius(f64) і Fahrenheit(f64) — обидва f64, але компілятор не дасть їх плутати.
struct Celsius(f64);
struct Fahrenheit(f64);

impl Celsius {
    fn to_fahrenheit(&self) -> Fahrenheit {
        Fahrenheit(self.0 * 9.0 / 5.0 + 32.0)
    }
}

impl Fahrenheit {
    fn value(&self) -> f64 {
        self.0
    }
}

fn lesson_tuple_struct() {
    let temp = Celsius(100.0);
    let converted = temp.to_fahrenheit();
    println!("{}°C = {}°F", temp.0, converted.value());

    // let wrong: Celsius = converted; // ← помилка: Fahrenheit != Celsius
    // Навіть якщо обидва містять f64, це різні типи.
}

// ============================================================
// ENUMS
// ============================================================

// --- Базовий enum: набір варіантів ---
// Як "одне з" — змінна може бути ОДНИМ з варіантів, не кількома одночасно.
// Усі варіанти — це один тип (Direction), тому можна передавати у функції.
#[derive(Debug)] // derive автоматично реалізує трейт Debug для {:?} виводу
#[allow(dead_code)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// --- match: вичерпний pattern matching ---
// match ВИМАГАЄ покрити ВСІ варіанти — компілятор перевіряє.
// Це захист від забутих випадків. Якщо додаси новий варіант в enum,
// компілятор скаже де ти забув його обробити.
fn describe_direction(dir: &Direction) -> &str {
    match dir {
        Direction::Up => "вгору ↑",
        Direction::Down => "вниз ↓",
        Direction::Left => "вліво ←",
        Direction::Right => "вправо →",
    }
}

fn lesson_enum() {
    let dir = Direction::Right;
    println!("напрямок: {:?} — {}", dir, describe_direction(&dir));

    // match з блоками коду
    let steps = match dir {
        Direction::Up | Direction::Down => 10, // | — "або"
        Direction::Left => 5,
        Direction::Right => 7,
    };
    println!("кроків: {steps}");
}

// --- Enum з даними (payload) ---
// Кожен варіант може містити різні дані — це потужніше за класичні enum з C/Java.
// Один варіант — tuple, інший — struct, третій — нічого.
#[derive(Debug)]
enum Command {
    Quit,                    // без даних
    Echo(String),            // один рядок
    Move { x: i32, y: i32 }, // іменовані поля (як struct)
    Color(u8, u8, u8),       // tuple-style
}

fn execute_command(cmd: &Command) {
    match cmd {
        Command::Quit => println!("  завершення"),
        Command::Echo(message) => println!("  echo: {message}"),
        Command::Move { x, y } => println!("  рух до ({x}, {y})"),
        Command::Color(r, g, b) => println!("  колір: rgb({r}, {g}, {b})"),
    }
}

fn lesson_enum_with_data() {
    let commands = [
        Command::Echo(String::from("hello")),
        Command::Move { x: 10, y: -5 },
        Command::Color(255, 128, 0),
        Command::Quit,
    ];

    for cmd in &commands {
        execute_command(cmd);
    }
}

// --- Option<T>: заміна null ---
// В Rust немає null/nil/None як "порожнього" значення змінної.
// Натомість є enum Option<T> з двома варіантами:
//   Some(T) — значення є
//   None    — значення немає
//
// Option вбудований у мову — не треба імпортувати, Some/None доступні глобально.
// Компілятор ЗМУШУЄ обробити None — забути перевірити неможливо.
fn find_first_negative(numbers: &[i32]) -> Option<i32> {
    for &n in numbers {
        if n < 0 {
            return Some(n);
        }
    }
    None
}

fn lesson_option() {
    let nums = [3, 7, -2, 5, -8];

    // match на Option
    match find_first_negative(&nums) {
        Some(n) => println!("знайшли від'ємне: {n}"),
        None => println!("від'ємних немає"),
    }

    // Без від'ємних
    let positive = [1, 2, 3];
    match find_first_negative(&positive) {
        Some(n) => println!("знайшли: {n}"),
        None => println!("від'ємних немає"),
    }

    // Корисні методи Option:
    let maybe: Option<i32> = Some(42);
    println!("unwrap_or: {}", maybe.unwrap_or(0)); // 42 — є значення
    println!("is_some: {}", maybe.is_some()); // true

    let empty: Option<i32> = None;
    println!("unwrap_or: {}", empty.unwrap_or(0)); // 0 — значення немає, fallback
    println!("is_none: {}", empty.is_none()); // true

    // .unwrap() — витягує значення АБО паніка якщо None.
    // Використовуй тільки коли 100% впевнений що Some, або в тестах.
    // В продакшн коді — match або unwrap_or.
}

// --- if let: скорочений match для одного варіанту ---
// Коли цікавить тільки один варіант, а решту ігноруємо.
// Замість повного match з _ => {} — пишемо if let.
fn lesson_if_let() {
    let config_value: Option<&str> = Some("dark");

    // Повний match — працює, але verbose для одного варіанту
    match config_value {
        Some(theme) => println!("тема (match): {theme}"),
        None => println!("тема не задана"),
    }

    // if let — те саме, коротше
    if let Some(theme) = config_value {
        println!("тема (if let): {theme}");
    } else {
        println!("тема не задана");
    }

    // if let з enum
    let cmd = Command::Move { x: 3, y: 7 };
    if let Command::Move { x, y } = &cmd {
        println!("рух: ({x}, {y})");
    }

    // while let — цикл поки pattern збігається
    // (корисно з ітераторами, тут простий приклад)
    let mut stack = vec![1, 2, 3]; // vec! — макрос для створення Vec<T>
    print!("stack: ");
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!();
}
