// ============================================================
// Урок 6 — Traits
// ============================================================
//
// Trait — набір методів, що описують спільну поведінку.
// Як інтерфейси (Go, Java), але потужніші:
// - можуть мати default реалізації
// - можна реалізувати для чужих типів
// - використовуються для operator overloading, конвертацій, форматування
//
// Trait — основа системи типів Rust. Все тримається на них:
// Display, Debug, Clone, Iterator, From, Into...

use std::fmt;

pub fn run() {
    println!("=== Trait — основи ===");
    lesson_trait_basics();

    println!("\n=== Default реалізація ===");
    lesson_default_impl();

    println!("\n=== Trait як параметр ===");
    lesson_trait_as_param();

    println!("\n=== Стандартні трейти: Display, PartialEq, PartialOrd ===");
    lesson_std_traits();

    println!("\n=== derive — автогенерація ===");
    lesson_derive();

    println!("\n=== From / Into — конвертації ===");
    lesson_from_into();

    println!("\n=== FromStr — парсинг з рядка ===");
    lesson_from_str();
}

// ============================================================
// TRAIT BASICS
// ============================================================

// Оголошуємо trait — "контракт" поведінки.
// Будь-який тип що реалізує Describable, зобов'язаний мати метод describe().
trait Describable {
    fn describe(&self) -> String;
}

// --- Два різні типи ---

struct Server {
    name: String,
    cpu_cores: u32,
    memory_gb: u32,
}

struct Process {
    pid: u32,
    name: String,
    cpu_usage: f64,
}

// --- Реалізуємо один trait для різних типів ---
// Кожен тип дає свою реалізацію. Trait гарантує що метод існує.

impl Describable for Server {
    fn describe(&self) -> String {
        format!("{}: {}C/{}GB", self.name, self.cpu_cores, self.memory_gb)
    }
}

impl Describable for Process {
    fn describe(&self) -> String {
        format!("[{}] {} ({:.1}% CPU)", self.pid, self.name, self.cpu_usage)
    }
}

fn lesson_trait_basics() {
    let server = Server {
        name: String::from("web-01"),
        cpu_cores: 8,
        memory_gb: 32,
    };

    let proc = Process {
        pid: 1234,
        name: String::from("nginx"),
        cpu_usage: 12.5,
    };

    // Обидва мають .describe(), хоч вони різні типи
    println!("  {}", server.describe());
    println!("  {}", proc.describe());
}

// ============================================================
// DEFAULT IMPLEMENTATION
// ============================================================
//
// Trait може мати реалізацію за замовчуванням.
// Тип може перевизначити (override), або використати default.

trait Loggable {
    fn name(&self) -> &str; // обов'язково реалізувати

    // Default — використовує name() всередині
    fn log(&self) {
        println!("  [LOG] {}", self.name());
    }

    // Default з кастомним форматом
    fn log_error(&self, msg: &str) {
        println!("  [ERROR] {}: {}", self.name(), msg);
    }
}

struct App {
    app_name: String,
}

// Реалізуємо тільки name(), решта — default
impl Loggable for App {
    fn name(&self) -> &str {
        &self.app_name
    }
}

struct Database {
    db_name: String,
}

// Реалізуємо name() + перевизначаємо log()
impl Loggable for Database {
    fn name(&self) -> &str {
        &self.db_name
    }

    fn log(&self) {
        println!("  [DB LOG] {} (connection pool active)", self.name());
    }
}

fn lesson_default_impl() {
    let app = App {
        app_name: String::from("api-server"),
    };
    let db = Database {
        db_name: String::from("postgres-main"),
    };

    app.log(); // default реалізація
    db.log(); // перевизначена

    app.log_error("timeout"); // обидва використовують default log_error
    db.log_error("connection lost");
}

// ============================================================
// TRAIT ЯК ПАРАМЕТР ФУНКЦІЇ
// ============================================================
//
// Можна вимагати "будь-який тип що реалізує trait X".
// Три синтаксиси — одне й те саме:
//
//   fn foo(item: &impl Describable)         // скорочений
//   fn foo<T: Describable>(item: &T)        // trait bound
//   fn foo<T>(item: &T) where T: Describable  // where clause

// --- impl Trait синтаксис (найпростіший) ---
fn print_description(item: &impl Describable) {
    println!("  → {}", item.describe());
}

// --- Trait bound (потрібен коли один тип в кількох місцях) ---
fn print_pair<T: Describable>(a: &T, b: &T) {
    println!("  пара: {} | {}", a.describe(), b.describe());
}

// --- Кілька trait bounds: тип має реалізувати ВСІ ---
fn log_and_describe(item: &(impl Describable + Loggable)) {
    item.log();
    println!("  опис: {}", item.describe());
}

// --- where clause — для складних bounds ---
fn process_items<T, U>(a: &T, b: &U)
where
    T: Describable + Loggable,
    U: Describable,
{
    println!("  a: {}", a.describe());
    println!("  b: {}", b.describe());
    a.log();
}

// Реалізуємо обидва трейти для Server щоб показати комбінацію
impl Loggable for Server {
    fn name(&self) -> &str {
        &self.name
    }
}

fn lesson_trait_as_param() {
    let server = Server {
        name: String::from("db-01"),
        cpu_cores: 16,
        memory_gb: 64,
    };
    let proc = Process {
        pid: 5678,
        name: String::from("redis"),
        cpu_usage: 3.2,
    };

    // impl Trait — приймає будь-який Describable
    print_description(&server);
    print_description(&proc);

    // Trait bound — обидва аргументи ОДНОГО типу T
    let server2 = Server {
        name: String::from("web-02"),
        cpu_cores: 4,
        memory_gb: 16,
    };
    print_pair(&server, &server2);

    // Кілька bounds — Server реалізує і Describable, і Loggable
    log_and_describe(&server);

    // where clause
    process_items(&server, &proc);
}

// ============================================================
// СТАНДАРТНІ ТРЕЙТИ
// ============================================================
//
// Rust має багато вбудованих трейтів. Найважливіші:
//   Display    — форматування для користувача (println!("{}", x))
//   Debug      — форматування для розробника (println!("{:?}", x))
//   Clone      — глибоке копіювання (.clone())
//   PartialEq  — порівняння == і !=
//   PartialOrd — порівняння < > <= >=

struct Temperature {
    celsius: f64,
    location: String,
}

// Display — ручна реалізація. Визначає як тип виглядає в println!("{}").
impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C ({})", self.celsius, self.location)
    }
}

// Debug — зазвичай через derive, але можна вручну
impl fmt::Debug for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Temperature {{ celsius: {}, location: {:?} }}",
            self.celsius, self.location
        )
    }
}

// PartialEq — порівнюємо тільки по температурі (без location)
impl PartialEq for Temperature {
    fn eq(&self, other: &Self) -> bool {
        self.celsius == other.celsius
    }
}

// PartialOrd — потребує PartialEq. Порівнюємо по celsius.
impl PartialOrd for Temperature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.celsius.partial_cmp(&other.celsius)
    }
}

fn lesson_std_traits() {
    let kyiv = Temperature {
        celsius: 22.5,
        location: String::from("Kyiv"),
    };
    let london = Temperature {
        celsius: 18.0,
        location: String::from("London"),
    };
    let paris = Temperature {
        celsius: 22.5,
        location: String::from("Paris"),
    };

    // Display — для користувача
    println!("  Display: {kyiv}");

    // Debug — для розробника
    println!("  Debug: {kyiv:?}");

    // PartialEq — порівнюємо по celsius
    println!("  Kyiv == Paris (by temp): {}", kyiv == paris);
    println!("  Kyiv == London: {}", kyiv == london);

    // PartialOrd
    println!("  Kyiv > London: {}", kyiv > london);
    println!("  London < Paris: {}", london < paris);
}

// ============================================================
// DERIVE — автогенерація трейтів
// ============================================================
//
// #[derive(...)] генерує стандартну реалізацію автоматично.
// Працює для: Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default

#[derive(Debug, Clone, PartialEq)]
struct Config {
    name: String,
    max_connections: u32,
    verbose: bool,
}

// Default — окремо, бо хочемо кастомні значення
impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            max_connections: 100,
            verbose: false,
        }
    }
}

fn lesson_derive() {
    let config1 = Config::default();
    println!("  default: {:?}", config1); // Debug через derive

    let config2 = config1.clone(); // Clone через derive
    println!("  clone: {:?}", config2);
    println!("  рівні: {}", config1 == config2); // PartialEq через derive

    // Struct update syntax + Default — задати тільки потрібні поля
    let custom = Config {
        name: String::from("production"),
        verbose: true,
        ..Config::default() // решту з default
    };
    println!("  custom: {:?}", custom);
    println!("  custom == default: {}", custom == config1);
}

// ============================================================
// FROM / INTO — конвертація між типами
// ============================================================
//
// From<T> — "як створити себе з типу T"
// Into<T> — автоматично реалізується якщо є From (зворотній)
//
// Це ідіоматичний Rust — замість конструкторів з різних типів.

#[derive(Debug)]
#[allow(dead_code)]
struct Milliseconds(u64);

#[derive(Debug)]
struct Seconds(f64);

// From<Seconds> для Milliseconds — конвертація секунд у мілісекунди
impl From<Seconds> for Milliseconds {
    fn from(s: Seconds) -> Self {
        Milliseconds((s.0 * 1000.0) as u64)
    }
}

// From<u64> — конвертація з простого числа
impl From<u64> for Milliseconds {
    fn from(ms: u64) -> Self {
        Milliseconds(ms)
    }
}

fn lesson_from_into() {
    // From — явна конвертація
    let duration = Milliseconds::from(Seconds(2.5));
    println!("  From: {:?}", duration); // Milliseconds(2500)

    // Into — автоматична зворотня. Rust виводить тип з контексту.
    let ms: Milliseconds = 500_u64.into();
    println!("  Into: {:?}", ms); // Milliseconds(500)

    // From для String — вже знайомий:
    let s: String = String::from("hello"); // From<&str> для String
    let s2: String = "world".into(); // Into автоматично
    println!("  String from/into: {s}, {s2}");

    // From часто використовується з ? для конвертації помилок
    show_from_for_errors();
}

fn show_from_for_errors() {
    // Якщо ErrorB реалізує From<ErrorA>, то ? автоматично конвертує
    // Це дозволяє ? працювати з різними типами помилок в одній функції
    println!("  (From для помилок — дозволяє ? конвертувати типи автоматично)");
}

// ============================================================
// FromStr — парсинг з рядка
// ============================================================
//
// impl FromStr дозволяє використовувати .parse::<MyType>()
// Саме цей трейт працює коли робиш "42".parse::<i32>()

use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl FromStr for LogLevel {
    type Err = String; // тип помилки парсингу

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warning" | "warn" => Ok(LogLevel::Warning),
            "error" | "err" => Ok(LogLevel::Error),
            other => Err(format!("unknown log level: '{other}'")),
        }
    }
}

// Display — щоб можна було і в зворотній бік (LogLevel → String)
impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

fn lesson_from_str() {
    // .parse() використовує FromStr
    let level: Result<LogLevel, _> = "warning".parse();
    println!("  parse \"warning\": {:?}", level);

    let level2: Result<LogLevel, _> = "INFO".parse();
    println!("  parse \"INFO\": {:?}", level2);

    let bad: Result<LogLevel, _> = "verbose".parse();
    println!("  parse \"verbose\": {:?}", bad);

    // Turbofish синтаксис — альтернативний спосіб вказати тип
    let level3 = "error".parse::<LogLevel>().unwrap();
    println!("  Display: {level3}"); // ERROR — через наш Display
}
