// ============================================================
// Урок 11 — Smart Pointers
// ============================================================
//
// Smart pointer — структура що поводиться як вказівник,
// але має додаткову семантику (підрахунок посилань, interior mutability...).
//
// Головні:
//   Box<T>      — heap allocation, одиночне ownership
//   Rc<T>       — reference counting, спільне ownership (single-thread)
//   RefCell<T>  — interior mutability (borrow checking at runtime)
//
// Трейти:
//   Deref  — дозволяє `*ptr` і автодеref
//   Drop   — деструктор (викликається при виході зі scope)

pub fn run() {
    println!("=== Box<T> — heap allocation ===");
    lesson_box();

    println!("\n=== Box<T> — рекурсивні типи ===");
    lesson_box_recursive();

    println!("\n=== Deref та автодеref ===");
    lesson_deref();

    println!("\n=== Drop — деструктор ===");
    lesson_drop();

    println!("\n=== Rc<T> — спільне ownership ===");
    lesson_rc();

    println!("\n=== RefCell<T> — interior mutability ===");
    lesson_refcell();

    println!("\n=== Rc<RefCell<T>> — спільне + змінне ===");
    lesson_rc_refcell();
}

// ============================================================
// Box<T> — HEAP ALLOCATION
// ============================================================
//
// Box<T> кладе T на heap, сам Box живе на стеку.
// При виході зі scope — автоматично звільняє heap пам'ять.
//
// Коли використовувати:
//   - Розмір типу невідомий на етапі компіляції (рекурсивні типи)
//   - Великі дані — щоб не копіювати стек при передачі
//   - Trait objects: Box<dyn Trait>

fn lesson_box() {
    // Прості значення на heap (на практиці не потрібно — але для демо)
    let b = Box::new(5);
    println!("  b = {b}"); // автодеref — не потрібно *b

    // Box автоматично деref'ується
    let x = Box::new(42);
    let y = *x + 1; // явний deref
    println!("  *x + 1 = {y}");

    // Передача у функцію — Box переміщується (move semantics)
    let boxed = Box::new(String::from("hello"));
    print_boxed(boxed);
    // println!("{boxed}"); // ПОМИЛКА: boxed переміщено

    // Box<dyn Trait> — зберігаємо різні типи за одним інтерфейсом
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 3.0 }),
        Box::new(Rectangle {
            width: 4.0,
            height: 5.0,
        }),
        Box::new(Circle { radius: 1.0 }),
    ];
    for shape in &shapes {
        println!("  area = {:.2}", shape.area());
    }
}

fn print_boxed(s: Box<String>) {
    println!("  boxed string: {s}");
}

trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

// ============================================================
// Box<T> — РЕКУРСИВНІ ТИПИ
// ============================================================
//
// Без Box рекурсивний тип має нескінченний розмір → не компілюється.
// Box має фіксований розмір (один вказівник) → розриває рекурсію.
//
// Без Box:
//   enum List { Cons(i32, List), Nil }  // ERROR: infinite size
//
// З Box:
//   enum List { Cons(i32, Box<List>), Nil }  // OK: ptr має фіксований розмір

#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> Self {
        List::Nil
    }

    fn prepend(self, value: i32) -> Self {
        List::Cons(value, Box::new(self))
    }

    fn sum(&self) -> i32 {
        match self {
            List::Cons(val, next) => val + next.sum(),
            List::Nil => 0,
        }
    }
}

fn lesson_box_recursive() {
    let list = List::new().prepend(3).prepend(2).prepend(1);

    println!("  list: {list:?}");
    println!("  sum: {}", list.sum());
}

// ============================================================
// DEREF — автоматичне розіменування
// ============================================================
//
// Deref trait дозволяє `*value` на власному типі.
// Rust застосовує deref coercion автоматично:
//   Box<String> → String → str
//   &String     → &str
//
// Це дозволяє передавати &Box<String> туди де очікується &str.

fn lesson_deref() {
    let boxed = Box::new(String::from("hello"));

    // Явний deref
    println!("  explicit: {}", *boxed);

    // Автодеref: Box<String> → String → str
    println_str(&boxed); // передаємо &Box<String>, функція очікує &str

    // Ланцюжок deref coercion:
    // &Box<String> → &String → &str — Rust робить це автоматично
    let s = String::from("world");
    let r = &s;
    println_str(r); // &String → &str автоматично
}

fn println_str(s: &str) {
    println!("  str: {s}");
}

// ============================================================
// DROP — деструктор
// ============================================================
//
// Drop trait: метод drop(&mut self) викликається коли значення
// виходить зі scope. Автоматично для Box, Rc, String, Vec...
//
// Можна реалізувати власний Drop для cleanup логіки.
// std::mem::drop(val) — явно скинути раніше ніж scope закінчиться.

struct Resource {
    name: String,
}

impl Drop for Resource {
    fn drop(&mut self) {
        println!("  [Drop] Resource '{}' звільнено", self.name);
    }
}

fn lesson_drop() {
    println!("  Створюємо r1 та r2...");
    let r1 = Resource {
        name: String::from("r1"),
    };
    let r2 = Resource {
        name: String::from("r2"),
    };

    println!("  Явно скидаємо r1 через drop()...");
    drop(r1); // r1 дропається тут, раніше кінця scope

    println!("  Кінець функції — r2 дропається автоматично:");
    let _ = r2; // просто щоб не було warning
    // r2 дропається тут — в порядку LIFO (зворотньому до створення)
}

// ============================================================
// Rc<T> — REFERENCE COUNTING
// ============================================================
//
// Rc<T> дозволяє КІЛЬКА власників одних даних.
// Кожен Rc::clone() збільшує лічильник на 1.
// При drop() лічильник зменшується. При 0 — дані звільняються.
//
// ТІЛЬКИ для single-threaded коду.
// Для потоків — Arc<T> (atomic reference counting).
//
// Rc дає тільки незмінний доступ (&T).
// Для змінного — Rc<RefCell<T>>.

use std::rc::Rc;

fn lesson_rc() {
    // Створення
    let a = Rc::new(String::from("shared data"));
    println!("  count після a: {}", Rc::strong_count(&a));

    // clone() НЕ копіює дані — збільшує лічильник
    let b = Rc::clone(&a);
    println!("  count після b: {}", Rc::strong_count(&a));

    {
        let c = Rc::clone(&a);
        println!("  count після c: {}", Rc::strong_count(&a));
        println!("  a={a}, b={b}, c={c} — всі вказують на одні дані");
    } // c дропається тут

    println!("  count після drop c: {}", Rc::strong_count(&a));

    // Практичний приклад — граф / дерево з кількома власниками
    let shared_config = Rc::new(vec![1, 2, 3]);
    let worker1 = Rc::clone(&shared_config);
    let worker2 = Rc::clone(&shared_config);

    println!("  worker1 бачить: {worker1:?}");
    println!("  worker2 бачить: {worker2:?}");
    println!(
        "  Це ті самі дані — count: {}",
        Rc::strong_count(&shared_config)
    );
}

// ============================================================
// RefCell<T> — INTERIOR MUTABILITY
// ============================================================
//
// Звичайне правило: або &T (кілька) або &mut T (одне) — вирішується
// на етапі компіляції.
//
// RefCell<T> переносить borrow checking на RUNTIME:
//   .borrow()     → Ref<T>      (незмінний доступ)
//   .borrow_mut() → RefMut<T>   (змінний доступ)
//
// Якщо правила порушені (два &mut одночасно) — паніка в runtime!
//
// Коли використовувати:
//   - Коли компілятор занадто консервативний
//   - Mock об'єкти в тестах
//   - Разом з Rc<T> для спільного змінного стану

use std::cell::RefCell;

fn lesson_refcell() {
    let data = RefCell::new(vec![1, 2, 3]);

    // Незмінний доступ
    println!("  data: {:?}", data.borrow());

    // Змінний доступ
    data.borrow_mut().push(4);
    println!("  after push: {:?}", data.borrow());

    // Кілька незмінних позичань одночасно — ОК
    let r1 = data.borrow();
    let r2 = data.borrow();
    println!("  r1: {r1:?}, r2: {r2:?}");
    drop(r1);
    drop(r2);

    // Zmінне після звільнення незмінних
    data.borrow_mut().sort_by(|a, b| b.cmp(a));
    println!("  sorted desc: {:?}", data.borrow());

    // ПАНІКА в runtime якщо порушити правила:
    // let _r = data.borrow();
    // let _rw = data.borrow_mut(); // panic! already borrowed
}

// ============================================================
// Rc<RefCell<T>> — СПІЛЬНЕ + ЗМІННЕ
// ============================================================
//
// Найпоширеніша комбінація:
//   Rc<T>         — кілька власників, тільки незмінний доступ
//   RefCell<T>    — один власник, змінний доступ в runtime
//   Rc<RefCell<T>> — кілька власників + змінний доступ

fn lesson_rc_refcell() {
    let shared = Rc::new(RefCell::new(0_i32));

    let clone1 = Rc::clone(&shared);
    let clone2 = Rc::clone(&shared);

    // Кожен клон може змінювати дані
    *clone1.borrow_mut() += 10;
    *clone2.borrow_mut() += 20;
    *shared.borrow_mut() += 5;

    println!("  shared value: {}", shared.borrow()); // 35

    // Практично: список задач що може редагуватися з кількох місць
    let todos: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    let editor1 = Rc::clone(&todos);
    let editor2 = Rc::clone(&todos);

    editor1.borrow_mut().push(String::from("Buy groceries"));
    editor2.borrow_mut().push(String::from("Write code"));
    todos.borrow_mut().push(String::from("Sleep"));

    println!("  todos: {:?}", todos.borrow());
    println!("  власників: {}", Rc::strong_count(&todos));
}
