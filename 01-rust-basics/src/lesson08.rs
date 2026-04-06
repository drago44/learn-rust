// ============================================================
// Урок 8 — Generics та Lifetimes
// ============================================================
//
// Generics — параметризація типів. Пишеш код один раз,
// працює з будь-яким типом що задовольняє trait bounds.
//
// Lifetimes — механізм що гарантує валідність посилань.
// Компілятор перевіряє що посилання не живе довше за дані.

pub fn run() {
    println!("=== Generic функції ===");
    lesson_generic_functions();

    println!("\n=== Generic struct та enum ===");
    lesson_generic_structs();

    println!("\n=== Trait bounds на generics ===");
    lesson_trait_bounds();

    println!("\n=== impl блоки з generics ===");
    lesson_generic_impl();

    println!("\n=== Lifetimes — основи ===");
    lesson_lifetime_basics();

    println!("\n=== Lifetime annotations у struct ===");
    lesson_lifetime_structs();

    println!("\n=== Lifetime elision — автовиведення ===");
    lesson_lifetime_elision();

    println!("\n=== 'static lifetime ===");
    lesson_static_lifetime();

    println!("\n=== Generics + Lifetimes разом ===");
    lesson_combined();
}

// ============================================================
// GENERIC ФУНКЦІЇ
// ============================================================
//
// <T> — параметр типу. Замість писати окремі функції для i32,
// f64, String — пишемо одну generic.

// Без generics — дублювання:
// fn largest_i32(list: &[i32]) -> &i32 { ... }
// fn largest_f64(list: &[f64]) -> &f64 { ... }

// З generics — одна функція для всіх типів що можна порівнювати
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut max = &list[0];
    for item in &list[1..] {
        if item > max {
            max = item;
        }
    }
    max
}

// Generic з кількома параметрами типу
fn pair_to_string<A: std::fmt::Display, B: std::fmt::Display>(a: A, b: B) -> String {
    format!("({a}, {b})")
}

fn lesson_generic_functions() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("  Largest i32: {}", largest(&numbers));

    let floats = vec![1.5, 3.7, 2.1, 0.8];
    println!("  Largest f64: {}", largest(&floats));

    let chars = vec!['a', 'z', 'm', 'b'];
    println!("  Largest char: {}", largest(&chars));

    // Кілька generic параметрів
    let result = pair_to_string(42, "hello");
    println!("  Pair: {result}");

    let result2 = pair_to_string(3.14, true);
    println!("  Pair: {result2}");
}

// ============================================================
// GENERIC STRUCT та ENUM
// ============================================================

// Struct з одним generic параметром
#[derive(Debug)]
struct Wrapper<T> {
    value: T,
    label: String,
}

// Struct з двома generic параметрами
#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V,
}

// Enum з generic — ти вже знаєш два найважливіші:
//   Option<T> = Some(T) | None
//   Result<T, E> = Ok(T) | Err(E)
//
// Власний generic enum:
#[derive(Debug)]
#[allow(dead_code)]
enum Response<T> {
    Success(T),
    Error(String),
    Loading,
}

fn lesson_generic_structs() {
    // Wrapper з різними типами
    let w1 = Wrapper {
        value: 42,
        label: String::from("answer"),
    };
    let w2 = Wrapper {
        value: "hello",
        label: String::from("greeting"),
    };
    println!("  {w1:?}");
    println!("  {w2:?}");

    // KeyValue
    let kv = KeyValue {
        key: "name",
        value: String::from("Rust"),
    };
    println!("  {kv:?}");

    // Generic enum
    let ok: Response<Vec<i32>> = Response::Success(vec![1, 2, 3]);
    let err: Response<Vec<i32>> = Response::Error(String::from("not found"));
    let loading: Response<String> = Response::Loading;
    println!("  {ok:?}");
    println!("  {err:?}");
    println!("  {loading:?}");
}

// ============================================================
// TRAIT BOUNDS — обмеження на generic типи
// ============================================================
//
// <T>               — будь-який тип (майже нічого не можна робити)
// <T: Display>      — тип що реалізує Display
// <T: Display + Clone>  — тип що реалізує обидва
// where T: Display  — альтернативний синтаксис

use std::fmt;

// Один trait bound
fn print_labeled<T: fmt::Display>(label: &str, value: T) {
    println!("  {label}: {value}");
}

// Кілька bounds
fn clone_and_print<T: fmt::Display + Clone>(value: &T) -> T {
    println!("  Cloning: {value}");
    value.clone()
}

// where clause — читабельніше при багатьох bounds
fn compare_and_show<T, U>(a: T, b: U) -> String
where
    T: fmt::Display + PartialOrd,
    U: fmt::Display,
{
    format!("{a} (compared with {b})")
}

// Повертаємо impl Trait — тип не називаємо, тільки поведінку
fn make_greeting(name: &str) -> impl fmt::Display {
    format!("Hello, {name}!")
}

fn lesson_trait_bounds() {
    print_labeled("Number", 42);
    print_labeled("Float", 3.14);

    let original = String::from("data");
    let cloned = clone_and_print(&original);
    println!("  Original: {original}, Cloned: {cloned}");

    let result = compare_and_show(10, "baseline");
    println!("  {result}");

    // impl Trait як return type
    let greeting = make_greeting("Rust");
    println!("  {greeting}");
}

// ============================================================
// IMPL з GENERICS
// ============================================================
//
// impl<T> дозволяє написати методи для generic struct.
// Можна додати bounds щоб методи працювали тільки для певних T.

impl<T: fmt::Display> Wrapper<T> {
    fn new(value: T, label: &str) -> Self {
        Self {
            value,
            label: label.to_string(),
        }
    }

    fn display(&self) -> String {
        format!("[{}] {}", self.label, self.value)
    }
}

// Спеціалізація — методи тільки для конкретного типу
impl Wrapper<f64> {
    fn rounded(&self) -> f64 {
        (self.value * 100.0).round() / 100.0
    }
}

impl<K: fmt::Display, V: fmt::Display> KeyValue<K, V> {
    fn format_pair(&self) -> String {
        format!("{} = {}", self.key, self.value)
    }
}

fn lesson_generic_impl() {
    let w = Wrapper::new(3.14159, "pi");
    println!("  {}", w.display());
    println!("  Rounded: {}", w.rounded()); // тільки для Wrapper<f64>

    let w2 = Wrapper::new("text", "type");
    println!("  {}", w2.display());
    // w2.rounded(); // ПОМИЛКА! rounded існує тільки для Wrapper<f64>

    let kv = KeyValue {
        key: "language",
        value: "Rust",
    };
    println!("  {}", kv.format_pair());
}

// ============================================================
// LIFETIMES — основи
// ============================================================
//
// Lifetime — час життя посилання. Компілятор перевіряє що
// посилання не переживає дані на які вказує.
//
// 'a — lifetime annotation. Не змінює час життя, а ОПИСУЄ
// зв'язок між lifetime'ами різних посилань.

// Без annotation — компілятор не знає який з двох &str повертається.
// Хто визначає як довго живе результат — x чи y?
// 'a каже: результат живе стільки, скільки коротший з двох.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

// Lifetime НЕ потрібен якщо повертається тільки один з параметрів
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

fn lesson_lifetime_basics() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
        // result валідний тут — обидва string ще живі
        println!("  Longest: {result}");
    }
    // Якби result використовувався тут — ПОМИЛКА!
    // string2 вже знищений, а result може вказувати на нього.
    // let _ = result; // ← не компілюється якщо result = string2

    let word = first_word("hello world");
    println!("  First word: {word}");

    // Lifetime з різними scope:
    let outer = String::from("outer lives longer");
    {
        let inner = String::from("inner");
        let chosen = longest(&outer, &inner);
        println!("  Chosen: {chosen}"); // ОК — обидва живі
    }
}

// ============================================================
// LIFETIME у STRUCT
// ============================================================
//
// Struct може зберігати посилання — але потрібен lifetime annotation.
// Це гарантує що struct не переживе дані на які посилається.

#[derive(Debug)]
#[allow(dead_code)]
struct Excerpt<'a> {
    text: &'a str,     // посилання на зовнішні дані
    page: u32,         // owned дані — lifetime не потрібен
}

impl<'a> Excerpt<'a> {
    fn new(text: &'a str, page: u32) -> Self {
        Self { text, page }
    }

    // Метод що повертає посилання з тим самим lifetime
    fn first_sentence(&self) -> &str {
        match self.text.find('.') {
            Some(i) => &self.text[..=i],
            None => self.text,
        }
    }
}

fn lesson_lifetime_structs() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let excerpt = Excerpt::new(&novel, 1);
    println!("  {:?}", excerpt);
    println!("  First sentence: {}", excerpt.first_sentence());

    // excerpt не може жити довше ніж novel:
    // {
    //     let short_lived = String::from("temp");
    //     let bad = Excerpt::new(&short_lived, 1);
    //     // bad не можна повернути чи зберегти за межами цього блоку
    // }
}

// ============================================================
// LIFETIME ELISION — автовиведення
// ============================================================
//
// Rust має правила elision — автоматичне виведення lifetime'ів.
// Не потрібно писати 'a вручну у більшості випадків:
//
// Правило 1: Кожен вхідний &параметр отримує свій lifetime
//   fn foo(x: &str, y: &str)  →  fn foo<'a, 'b>(x: &'a str, y: &'b str)
//
// Правило 2: Якщо один вхідний lifetime — він же стає вихідним
//   fn foo(x: &str) -> &str   →  fn foo<'a>(x: &'a str) -> &'a str
//
// Правило 3: Якщо є &self — lifetime self стає вихідним
//   fn method(&self) -> &str  →  fn method<'a>(&'a self) -> &'a str

// Правило 2: один вхід → автоматично
fn trim_and_lower(s: &str) -> &str {
    s.trim()
    // Rust виводить: fn trim_and_lower<'a>(s: &'a str) -> &'a str
}

// Правила НЕ працюють для двох вхідних посилань і одного вихідного
// fn ambiguous(a: &str, b: &str) -> &str { a }  // ПОМИЛКА!
// Потрібно явно: fn ambiguous<'a>(a: &'a str, _b: &str) -> &'a str

fn lesson_lifetime_elision() {
    let padded = String::from("  hello  ");
    let trimmed = trim_and_lower(&padded);
    println!("  Trimmed: '{trimmed}'");

    println!("  Правила elision:");
    println!("    1. Кожен &параметр → свій lifetime");
    println!("    2. Один вхідний lifetime → він же вихідний");
    println!("    3. &self → його lifetime стає вихідним");
    println!("  Якщо правила не вирішують — пиши 'a вручну");
}

// ============================================================
// 'STATIC LIFETIME
// ============================================================
//
// 'static — спеціальний lifetime: дані живуть весь час програми.
//
// Два випадки:
// 1. Рядкові літерали: "hello" має тип &'static str
//    (вшито в бінарник, живе завжди)
// 2. Owned типи: String, Vec<T> можна конвертувати в 'static
//    бо вони самодостатні (не посилаються на тимчасові дані)

fn lesson_static_lifetime() {
    // Рядковий літерал — завжди 'static
    let s: &'static str = "I live forever";
    println!("  Static str: {s}");

    // Константи — теж 'static
    const MAX_SIZE: u32 = 1024;
    println!("  Const: {MAX_SIZE}");

    // УВАГА: T: 'static НЕ означає "живе вічно"!
    // Означає: "не містить посилань з обмеженим lifetime"
    // String: 'static — бо не посилається ні на що тимчасове
    // &'a str: НЕ 'static (якщо 'a != 'static) — посилається на щось

    fn needs_static<T: std::fmt::Display + 'static>(val: T) {
        println!("  'static value: {val}");
    }

    needs_static(String::from("owned String is 'static"));
    needs_static(42); // i32 — Copy, немає посилань → 'static
    needs_static("literal"); // &'static str — ок

    // needs_static(&local_string); // ПОМИЛКА якщо local_string не 'static
}

// ============================================================
// GENERICS + LIFETIMES разом
// ============================================================

#[derive(Debug)]
struct Cache<'a, T> {
    data: &'a [T],    // посилання на зріз — потрібен lifetime
    name: String,      // owned — lifetime не потрібен
}

impl<'a, T: fmt::Display> Cache<'a, T> {
    fn new(data: &'a [T], name: &str) -> Self {
        Self {
            data,
            name: name.to_string(),
        }
    }

    fn first(&self) -> Option<&T> {
        self.data.first()
    }

    fn summary(&self) -> String {
        format!("{}: {} items", self.name, self.data.len())
    }
}

// Функція з generic + lifetime + trait bound
fn longest_displayed<'a, T: fmt::Display>(items: &'a [T]) -> Option<String> {
    items.iter().map(|item| format!("{item}")).max_by_key(|s| s.len())
}

fn lesson_combined() {
    let numbers = vec![10, 20, 30, 40, 50];
    let cache = Cache::new(&numbers, "numbers");
    println!("  {}", cache.summary());
    println!("  First: {:?}", cache.first());

    let words = vec!["short", "much longer text", "mid"];
    let longest = longest_displayed(&words);
    println!("  Longest display: {:?}", longest);

    let cache2 = Cache::new(&words, "words");
    println!("  {}", cache2.summary());
    println!("  {:?}", cache2);
}
