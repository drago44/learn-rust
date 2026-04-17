// ============================================================
// Урок 14 — Pattern Matching поглиблено та Advanced
// ============================================================
//
// Pattern matching в Rust — не просто switch.
// Patterns можна використовувати в:
//   match, if let, while let, let, for, fn параметри

pub fn run() {
    println!("=== Деструктуризація struct ===");
    lesson_destructure_struct();

    println!("\n=== Деструктуризація enum ===");
    lesson_destructure_enum();

    println!("\n=== Деструктуризація tuple та nested ===");
    lesson_destructure_tuple_nested();

    println!("\n=== Match guards ===");
    lesson_match_guards();

    println!("\n=== @ bindings ===");
    lesson_at_bindings();

    println!("\n=== Patterns у let, for, fn ===");
    lesson_patterns_everywhere();

    println!("\n=== Unsafe Rust — огляд ===");
    lesson_unsafe_overview();

    println!("\n=== macro_rules! — основи ===");
    lesson_macros();
}

// ============================================================
// ДЕСТРУКТУРИЗАЦІЯ STRUCT
// ============================================================
//
// struct можна деструктуризувати у match, let, fn параметрах.
// `..` — ігнорує решту полів.

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug)]
struct Rect {
    top_left: Point,
    bottom_right: Point,
    #[allow(dead_code)]
    color: Color,
}

fn lesson_destructure_struct() {
    let p = Point { x: 3.0, y: 7.0 };

    // Деструктуризація в let
    let Point { x, y } = p;
    println!("  x={x}, y={y}");

    // Деструктуризація з перейменуванням
    let p2 = Point { x: 1.0, y: 5.0 };
    let Point { x: px, y: py } = p2;
    println!("  px={px}, py={py}");

    // Деструктуризація в match
    let color = Color { r: 255, g: 128, b: 0 };
    match color {
        Color { r: 255, g, b: 0 } => println!("  Червоно-зелений: g={g}"),
        Color { r: 0, g: 0, b } => println!("  Синій: b={b}"),
        Color { r, g, b } => println!("  Змішаний: {r},{g},{b}"),
    }

    // `..` — пропускаємо решту полів
    let rect = Rect {
        top_left: Point { x: 0.0, y: 10.0 },
        bottom_right: Point { x: 10.0, y: 0.0 },
        color: Color { r: 100, g: 200, b: 50 },
    };
    let Rect { top_left: Point { x: x1, .. }, bottom_right: Point { x: x2, .. }, .. } = rect;
    println!("  ширина rect: {}", x2 - x1);
}

// ============================================================
// ДЕСТРУКТУРИЗАЦІЯ ENUM
// ============================================================

#[derive(Debug)]
#[allow(dead_code)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

#[derive(Debug)]
#[allow(dead_code)]
enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle(Rect),
    Triangle(Point, Point, Point),
}

fn lesson_destructure_enum() {
    let messages = vec![
        Message::Quit,
        Message::Move { x: 10, y: 20 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(255, 128, 0),
    ];

    for msg in &messages {
        match msg {
            Message::Quit => println!("  Quit"),
            Message::Move { x, y } => println!("  Move до ({x}, {y})"),
            Message::Write(text) => println!("  Write: {text}"),
            Message::ChangeColor(r, g, b) => println!("  Color: rgb({r},{g},{b})"),
        }
    }

    // Nested enum деструктуризація
    let shape = Shape::Circle {
        center: Point { x: 0.0, y: 0.0 },
        radius: 5.0,
    };

    match shape {
        Shape::Circle { center: Point { x, y }, radius } => {
            println!("  Коло: центр ({x},{y}), r={radius}");
        }
        Shape::Rectangle(Rect { top_left: Point { x, y }, .. }) => {
            println!("  Прямокутник від ({x},{y})");
        }
        Shape::Triangle(p1, p2, p3) => {
            println!("  Трикутник: {p1:?}, {p2:?}, {p3:?}");
        }
    }
}

// ============================================================
// ДЕСТРУКТУРИЗАЦІЯ TUPLE та NESTED
// ============================================================

fn lesson_destructure_tuple_nested() {
    // Tuple деструктуризація
    let (a, b, c) = (1, "hello", 3.14);
    println!("  a={a}, b={b}, c={c}");

    // Вкладені tuple
    let ((x1, y1), (x2, y2)) = ((0, 0), (10, 10));
    println!("  від ({x1},{y1}) до ({x2},{y2})");

    // `_` — ігнорує одне значення
    let (first, _, third) = (1, 2, 3);
    println!("  first={first}, third={third}");

    // `..` — ігнорує кілька
    let (head, .., tail) = (1, 2, 3, 4, 5);
    println!("  head={head}, tail={tail}");

    // Деструктуризація в for
    let points = vec![(1, 2), (3, 4), (5, 6)];
    for (x, y) in &points {
        println!("  point: ({x},{y})");
    }

    // Деструктуризація пар з enumerate
    let words = vec!["alpha", "beta", "gamma"];
    for (i, word) in words.iter().enumerate() {
        println!("  [{i}] {word}");
    }

    // Nested struct + tuple
    let data: Vec<(Point, &str)> = vec![
        (Point { x: 1.0, y: 2.0 }, "A"),
        (Point { x: 3.0, y: 4.0 }, "B"),
    ];
    for (Point { x, y }, label) in &data {
        println!("  {label}: ({x},{y})");
    }
}

// ============================================================
// MATCH GUARDS
// ============================================================
//
// Match guard — додаткова умова після pattern: `pattern if condition`
// Guard обчислюється тільки якщо pattern зіставився.
// Компілятор НЕ перевіряє вичерпність з урахуванням guards!

fn lesson_match_guards() {
    // Базовий guard
    let num = 7;
    match num {
        n if n < 0 => println!("  {n} від'ємне"),
        n if n % 2 == 0 => println!("  {n} парне"),
        n => println!("  {n} непарне додатне"),
    }

    // Guard з деструктуризацією
    let pair = (2, -3);
    match pair {
        (x, y) if x == y => println!("  рівні: {x}"),
        (x, y) if x + y == 0 => println!("  сума нуль: {x} + {y}"),
        (x, y) if x > 0 && y > 0 => println!("  обидва додатні: {x},{y}"),
        (x, y) => println!("  решта: {x},{y}"),
    }

    // Guard з enum
    let msg = Message::Move { x: -5, y: 10 };
    match msg {
        Message::Move { x, y } if x < 0 => println!("  рух вліво: x={x}, y={y}"),
        Message::Move { x, y } if y < 0 => println!("  рух вниз: x={x}, y={y}"),
        Message::Move { x, y } => println!("  рух: x={x}, y={y}"),
        _ => println!("  інше"),
    }

    // Guard з Option
    let values = vec![Some(1), None, Some(15), Some(7), None];
    for val in &values {
        match val {
            Some(n) if *n > 10 => println!("  велике: {n}"),
            Some(n) => println!("  мале: {n}"),
            None => println!("  відсутнє"),
        }
    }
}

// ============================================================
// @ BINDINGS
// ============================================================
//
// `@` — зберігає значення в змінну ОДНОЧАСНО з перевіркою pattern.
// Синтаксис: `name @ pattern`
//
// Без @: або перевіряємо діапазон (але не маємо значення),
//        або зберігаємо (але не можемо обмежити діапазон).

fn lesson_at_bindings() {
    // Базовий @ binding
    let num = 7_u32;
    match num {
        n @ 1..=10 => println!("  {n} в діапазоні 1..=10"),
        n @ 11..=20 => println!("  {n} в діапазоні 11..=20"),
        n => println!("  {n} поза діапазонами"),
    }

    // @ з enum
    #[derive(Debug)]
    enum Temperature {
        Celsius(f64),
        Fahrenheit(f64),
    }

    let temps = vec![
        Temperature::Celsius(-10.0),
        Temperature::Celsius(20.0),
        Temperature::Celsius(40.0),
        Temperature::Fahrenheit(32.0),
    ];

    for temp in &temps {
        match temp {
            Temperature::Celsius(t @ ..=-0.1) => println!("  мороз: {t}°C"),
            Temperature::Celsius(t @ 0.0..=25.0) => println!("  комфорт: {t}°C"),
            Temperature::Celsius(t) => println!("  спека: {t}°C"),
            Temperature::Fahrenheit(f) => println!("  {f}°F"),
        }
    }

    // @ з деструктуризацією struct
    let points = vec![
        Point { x: 0.0, y: 0.0 },
        Point { x: 3.0, y: 4.0 },
        Point { x: 10.0, y: 0.0 },
    ];

    for p in &points {
        match p {
            Point { x: 0.0, y: 0.0 } => println!("  початок координат"),
            Point { x, y: 0.0 } => println!("  на осі X: x={x}"),
            pt @ Point { x, y } => {
                let dist = (x * x + y * y).sqrt();
                println!("  {pt:?}, відстань від 0: {dist:.2}");
            }
        }
    }
}

// ============================================================
// PATTERNS У let, for, fn
// ============================================================

fn sum_pair((a, b): (i32, i32)) -> i32 {
    a + b
}

fn first_and_rest(slice: &[i32]) -> Option<(i32, &[i32])> {
    match slice {
        [] => None,
        [first, rest @ ..] => Some((*first, rest)),
    }
}

fn lesson_patterns_everywhere() {
    // Pattern у fn параметрі
    println!("  sum_pair: {}", sum_pair((3, 4)));

    // if let — скорочений match для одного варіанту
    let config: Option<&str> = Some("debug");
    if let Some(level) = config {
        println!("  log level: {level}");
    }

    // if let + else
    let value: Result<i32, &str> = Ok(42);
    if let Ok(n) = value {
        println!("  ok: {n}");
    } else {
        println!("  error");
    }

    // while let — цикл поки pattern зіставляється
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("  popped: {top}");
    }

    // Slice patterns
    let numbers = vec![1, 2, 3, 4, 5];
    match numbers.as_slice() {
        [] => println!("  порожній"),
        [single] => println!("  один: {single}"),
        [first, second, ..] => println!("  починається з {first}, {second}"),
    }

    // first_and_rest рекурсивно
    let data = [10, 20, 30, 40];
    if let Some((head, tail)) = first_and_rest(&data) {
        println!("  head={head}, tail={tail:?}");
    }
}

// ============================================================
// UNSAFE RUST — ОГЛЯД
// ============================================================
//
// `unsafe` — блок де вимикаються певні перевірки компілятора.
// Rust гарантує memory safety ТІЛЬКИ в safe коді.
//
// П'ять речей що можна тільки в unsafe:
//   1. Розіменовувати raw pointer (*const T, *mut T)
//   2. Викликати unsafe функції
//   3. Реалізовувати unsafe trait (Send, Sync вручну)
//   4. Мутувати static змінні
//   5. Читати union поля
//
// unsafe НЕ вимикає borrow checker повністю!
// Він тільки дозволяє ці п'ять операцій.

fn lesson_unsafe_overview() {
    // Raw pointers — можна створювати в safe коді
    let x = 42_i32;
    let raw_const: *const i32 = &x;
    let mut y = 10_i32;
    let raw_mut: *mut i32 = &mut y;

    // Розіменування — тільки в unsafe
    unsafe {
        println!("  *raw_const = {}", *raw_const);
        *raw_mut = 99;
        println!("  *raw_mut після запису = {}", *raw_mut);
    }
    println!("  y після unsafe = {y}");

    // unsafe функція
    unsafe fn dangerous(ptr: *const i32) -> i32 {
        unsafe { *ptr }
    }

    let result = unsafe { dangerous(raw_const) };
    println!("  unsafe fn result = {result}");

    // Типовий use case: FFI (виклик C функцій)
    // extern "C" { fn abs(x: i32) -> i32; }
    // unsafe { abs(-5) }

    println!("  unsafe використовується для: FFI, raw pointers,");
    println!("  zero-cost abstractions (Vec::from_raw_parts, etc.)");
    println!("  Ізолюй unsafe в мінімальних блоках з чіткими інваріантами.");
}

// ============================================================
// macro_rules! — ДЕКЛАРАТИВНІ МАКРОСИ
// ============================================================
//
// Макрос — код що генерує код на етапі компіляції.
// macro_rules! — pattern matching над синтаксичними деревами.
//
// Синтаксис:
//   macro_rules! name {
//       (pattern) => { expansion };
//       (pattern) => { expansion };
//   }
//
// Метазмінні: $name:kind
//   expr  — вираз
//   ident — ідентифікатор
//   ty    — тип
//   pat   — pattern
//   stmt  — оператор
//   block — блок {}
//   tt    — token tree (будь-що)
//   literal — літерал
//
// Повторення: $(...),* або $(...),+

macro_rules! say_hello {
    () => {
        println!("  Hello from macro!");
    };
    ($name:expr) => {
        println!("  Hello, {}!", $name);
    };
}

macro_rules! create_vec {
    ($($elem:expr),*) => {
        {
            let mut v = Vec::new();
            $(v.push($elem);)*
            v
        }
    };
}

macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = ($left - $right).abs();
        assert!(
            diff < $epsilon,
            "assert_approx_eq failed: |{} - {}| = {} >= {}",
            $left, $right, diff, $epsilon
        );
    };
}

macro_rules! log {
    ($level:ident, $($arg:tt)*) => {
        println!("  [{}] {}", stringify!($level), format!($($arg)*));
    };
}

fn lesson_macros() {
    // Базовий макрос з кількома варіантами
    say_hello!();
    say_hello!("Rust");

    // Макрос що генерує Vec
    let v = create_vec![1, 2, 3, 4, 5];
    println!("  create_vec: {v:?}");

    // Схожий до вбудованого vec![]
    let v2 = vec![10, 20, 30];
    println!("  vec!: {v2:?}");

    // Макрос для float порівняння (assert_eq! не працює через floating point)
    assert_approx_eq!(3.14159_f64, std::f64::consts::PI, 0.001);
    println!("  assert_approx_eq PI passed");

    // Макрос з $level:ident + $($arg:tt)*
    log!(INFO, "server started on port {}", 8080);
    log!(WARN, "memory usage: {}%", 85);
    log!(ERROR, "connection failed");

    // stringify! — вбудований макрос, перетворює токени в рядок
    let code = stringify!(let x = 1 + 2);
    println!("  stringify: {code}");

    // Різниця макрос vs функція:
    // vec![1, 2, 3]     — макрос, довільна кількість аргументів
    // println!("{}", x) — макрос, форматний рядок перевіряється при компіляції
    // fn max(a, b)       — функція, фіксована кількість аргументів одного типу
}
