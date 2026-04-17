// ============================================================
// Урок 13 — Trait Objects та OOP-патерни
// ============================================================
//
// Rust не має класів та наслідування, але підтримує:
//   - Інкапсуляцію через struct + impl
//   - Поліморфізм через trait objects (dyn Trait)
//   - Композицію замість наслідування
//
// Два види поліморфізму:
//   Static dispatch  — impl Trait / generics — розкривається на етапі компіляції
//   Dynamic dispatch — dyn Trait — вирішується в runtime через vtable

pub fn run() {
    println!("=== dyn Trait — trait objects ===");
    lesson_dyn_trait();

    println!("\n=== Box<dyn Trait> — колекція різних типів ===");
    lesson_box_dyn();

    println!("\n=== static vs dynamic dispatch ===");
    lesson_dispatch_comparison();

    println!("\n=== Object safety — обмеження dyn ===");
    lesson_object_safety();

    println!("\n=== State pattern ===");
    lesson_state_pattern();

    println!("\n=== Strategy pattern ===");
    lesson_strategy_pattern();

    println!("\n=== Builder pattern ===");
    lesson_builder_pattern();
}

// ============================================================
// dyn Trait — TRAIT OBJECTS
// ============================================================
//
// `dyn Trait` — тип що позначає "будь-який тип що реалізує Trait".
// Розмір невідомий на етапі компіляції → завжди за вказівником:
//   &dyn Trait
//   Box<dyn Trait>
//   Arc<dyn Trait>
//
// Fat pointer: (вказівник на дані, вказівник на vtable)
// vtable — таблиця функцій для конкретного типу.

trait Animal {
    fn name(&self) -> &str;
    fn sound(&self) -> &str;
    fn describe(&self) -> String {
        format!("{} каже {}", self.name(), self.sound())
    }
}

struct Dog {
    name: String,
}

struct Cat {
    name: String,
}

struct Cow;

impl Animal for Dog {
    fn name(&self) -> &str { &self.name }
    fn sound(&self) -> &str { "Гав!" }
}

impl Animal for Cat {
    fn name(&self) -> &str { &self.name }
    fn sound(&self) -> &str { "Няв!" }
}

impl Animal for Cow {
    fn name(&self) -> &str { "Корова" }
    fn sound(&self) -> &str { "Му!" }
}

fn lesson_dyn_trait() {
    // &dyn Trait — позичений trait object
    let dog = Dog { name: String::from("Рекс") };
    let cat = Cat { name: String::from("Мурка") };

    let animal: &dyn Animal = &dog;
    println!("  {}", animal.describe());

    // Функція що приймає будь-який Animal
    print_animal(&cat);
    print_animal(&Cow);
}

fn print_animal(animal: &dyn Animal) {
    println!("  {}", animal.describe());
}

// ============================================================
// Box<dyn Trait> — ГЕТЕРОГЕННІ КОЛЕКЦІЇ
// ============================================================
//
// Vec<Box<dyn Trait>> — вектор різних типів що реалізують Trait.
// Без dyn неможливо: Vec<Dog> і Vec<Cat> — різні типи.

fn lesson_box_dyn() {
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog { name: String::from("Барс") }),
        Box::new(Cat { name: String::from("Сірко") }),
        Box::new(Cow),
        Box::new(Dog { name: String::from("Жук") }),
    ];

    for animal in &animals {
        println!("  {}", animal.describe());
    }

    // Фільтрація через метод трейту
    let loud: Vec<&Box<dyn Animal>> = animals
        .iter()
        .filter(|a| a.sound().ends_with('!'))
        .collect();
    println!("  Гучних тварин: {}", loud.len());
}

// ============================================================
// STATIC vs DYNAMIC DISPATCH
// ============================================================
//
// Static (impl Trait / generics):
//   + швидше — нема vtable lookup
//   + компілятор може інлайнити
//   - бінарник більший (monomorphization — окремий код для кожного типу)
//   - не можна зберегти різні типи разом
//
// Dynamic (dyn Trait):
//   + гнучкість — різні типи в одній колекції
//   + менший бінарник
//   - vtable lookup при кожному виклику
//   - не всі трейти є "object-safe"

// Static dispatch — компілятор генерує окрему версію для Dog і Cat
fn static_describe(animal: &impl Animal) -> String {
    animal.describe()
}

// Dynamic dispatch — одна функція, vtable вирішує яку реалізацію викликати
fn dynamic_describe(animal: &dyn Animal) -> String {
    animal.describe()
}

fn lesson_dispatch_comparison() {
    let dog = Dog { name: String::from("Пес") };
    let cat = Cat { name: String::from("Кіт") };

    // static: компілятор розкриє в дві функції
    println!("  static: {}", static_describe(&dog));
    println!("  static: {}", static_describe(&cat));

    // dynamic: одна функція, vtable вирішує
    println!("  dynamic: {}", dynamic_describe(&dog));
    println!("  dynamic: {}", dynamic_describe(&cat));

    // Generic з trait bound = static dispatch
    fn generic_print<T: Animal>(a: &T) {
        println!("  generic: {}", a.describe());
    }
    generic_print(&dog);
    generic_print(&cat);
}

// ============================================================
// OBJECT SAFETY
// ============================================================
//
// Не кожен trait можна використати як dyn Trait.
// Trait є "object-safe" якщо:
//   1. Методи не повертають Self
//   2. Методи не мають generic параметрів
//   3. Немає associated functions (без self)
//
// Clone не є object-safe (повертає Self).
// Display не є object-safe (метод fmt має &Formatter).
// Але можна обійти через wrapper або додатковий трейт.

trait Printable {
    fn print(&self);
}

trait Cloneable: Printable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Printable for Point {
    fn print(&self) {
        println!("  Point({}, {})", self.x, self.y);
    }
}

impl Cloneable for Point {
    fn clone_box(&self) -> Box<dyn Cloneable> {
        Box::new(self.clone())
    }
}

fn lesson_object_safety() {
    let p = Point { x: 1.0, y: 2.0 };
    let boxed: Box<dyn Printable> = Box::new(p.clone());
    boxed.print();

    // Clone через wrapper — обійшли обмеження object safety
    let cloneable: Box<dyn Cloneable> = Box::new(p);
    let cloned = cloneable.clone_box();
    cloned.print();
}

// ============================================================
// STATE PATTERN
// ============================================================
//
// Об'єкт змінює поведінку залежно від внутрішнього стану.
// В Rust: замість мутувати поле state — повертаємо новий об'єкт.
// Тип виражає стан → неможливо зробити неправильний виклик.

struct Draft {
    content: String,
}

struct PendingReview {
    content: String,
}

struct Published {
    content: String,
}

impl Draft {
    fn new(content: &str) -> Self {
        Self { content: content.to_string() }
    }

    fn request_review(self) -> PendingReview {
        PendingReview { content: self.content }
    }
}

impl PendingReview {
    fn approve(self) -> Published {
        Published { content: self.content }
    }

    fn reject(self) -> Draft {
        Draft { content: self.content }
    }
}

impl Published {
    fn content(&self) -> &str {
        &self.content
    }
}

fn lesson_state_pattern() {
    let post = Draft::new("Мій перший пост про Rust");

    // draft.content() — немає такого методу! Тип не дозволяє.
    let post = post.request_review();
    // post.approve_without_review() — теж немає.
    let post = post.approve();

    println!("  Опублікований пост: {}", post.content());

    // Відхилений пост повертається в Draft
    let draft2 = Draft::new("Чернетка");
    let pending = draft2.request_review();
    let draft_again = pending.reject();
    let _published = draft_again.request_review().approve();
    println!("  Відхилено і переопубліковано — ОК");
}

// ============================================================
// STRATEGY PATTERN
// ============================================================
//
// Алгоритм передається як параметр (closure або trait object).
// Дозволяє змінювати поведінку без зміни структури.

trait SortStrategy {
    fn sort(&self, data: &mut Vec<i32>);
    fn name(&self) -> &str;
}

struct AscendingSort;
struct DescendingSort;
struct AbsoluteSort;

impl SortStrategy for AscendingSort {
    fn sort(&self, data: &mut Vec<i32>) { data.sort(); }
    fn name(&self) -> &str { "за зростанням" }
}

impl SortStrategy for DescendingSort {
    fn sort(&self, data: &mut Vec<i32>) { data.sort_by(|a, b| b.cmp(a)); }
    fn name(&self) -> &str { "за спаданням" }
}

impl SortStrategy for AbsoluteSort {
    fn sort(&self, data: &mut Vec<i32>) { data.sort_by_key(|x| x.abs()); }
    fn name(&self) -> &str { "за модулем" }
}

struct Sorter {
    strategy: Box<dyn SortStrategy>,
}

impl Sorter {
    fn new(strategy: Box<dyn SortStrategy>) -> Self {
        Self { strategy }
    }

    fn sort(&self, data: &mut Vec<i32>) {
        self.strategy.sort(data);
        println!("  сортування {}: {data:?}", self.strategy.name());
    }
}

fn lesson_strategy_pattern() {
    let mut data = vec![3, -1, 4, -1, 5, -9, 2, 6];

    Sorter::new(Box::new(AscendingSort)).sort(&mut data.clone());
    Sorter::new(Box::new(DescendingSort)).sort(&mut data.clone());
    Sorter::new(Box::new(AbsoluteSort)).sort(&mut data);
}

// ============================================================
// BUILDER PATTERN
// ============================================================
//
// Покроково конструює складний об'єкт.
// Кожен метод повертає Self → ланцюжкові виклики.
// Фінальний .build() валідує і повертає результат.

#[derive(Debug)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: u64,
}

struct HttpRequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: u64,
}

impl HttpRequestBuilder {
    fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_uppercase(),
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
            timeout_ms: 5000,
        }
    }

    fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    fn build(self) -> Result<HttpRequest, String> {
        if self.url.is_empty() {
            return Err(String::from("URL не може бути порожнім"));
        }
        Ok(HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
            timeout_ms: self.timeout_ms,
        })
    }
}

fn lesson_builder_pattern() {
    let request = HttpRequestBuilder::new("post", "https://api.example.com/data")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token123")
        .body(r#"{"key": "value"}"#)
        .timeout(3000)
        .build()
        .unwrap();

    println!("  {} {}", request.method, request.url);
    println!("  headers: {:?}", request.headers);
    println!("  body: {:?}", request.body);
    println!("  timeout: {}ms", request.timeout_ms);

    // Мінімальний запит
    let get = HttpRequestBuilder::new("get", "https://api.example.com/users")
        .build()
        .unwrap();
    println!("  {} {} (default timeout: {}ms)", get.method, get.url, get.timeout_ms);

    // Валідація
    let err = HttpRequestBuilder::new("get", "").build();
    println!("  порожній URL: {:?}", err);
}
