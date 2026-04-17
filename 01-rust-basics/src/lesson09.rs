// ============================================================
// Урок 9 — Closures та Iterators
// ============================================================
//
// Closure — анонімна функція що захоплює змінні з навколишнього scope.
// Iterator — абстракція над послідовністю значень, лінива.
//
// Разом вони утворюють функціональний стиль Rust:
// data.iter().filter(|x| ...).map(|x| ...).collect()

pub fn run() {
    println!("=== Closure — базовий синтаксис ===");
    lesson_closure_basics();

    println!("\n=== Closure захоплення змінних ===");
    lesson_closure_capture();

    println!("\n=== Fn, FnMut, FnOnce ===");
    lesson_closure_traits();

    println!("\n=== move closure ===");
    lesson_move_closure();

    println!("\n=== Iterator — базові адаптери ===");
    lesson_iterator_adapters();

    println!("\n=== Iterator — consumers ===");
    lesson_iterator_consumers();

    println!("\n=== iter() vs into_iter() vs iter_mut() ===");
    lesson_iter_variants();

    println!("\n=== flat_map, zip, chain ===");
    lesson_iterator_advanced();

    println!("\n=== Власний Iterator ===");
    lesson_custom_iterator();
}

// ============================================================
// CLOSURE — БАЗОВИЙ СИНТАКСИС
// ============================================================
//
// Closure = анонімна функція що "закриває" над змінними scope.
//
// Три форми запису:
//   |x: i32| -> i32 { x + 1 }   // повний
//   |x: i32| x + 1               // скорочений (без {})
//   |x| x + 1                    // без типів (виводяться)
//
// Типи виводяться при першому виклику і фіксуються.

fn lesson_closure_basics() {
    // Звичайна функція
    fn add_one_fn(x: i32) -> i32 {
        x + 1
    }

    // Closure — те саме, але зберігається в змінній
    let add_one = |x: i32| -> i32 { x + 1 };
    let add_one_short = |x| x + 1; // типи виводяться

    println!("  fn:      {}", add_one_fn(5));
    println!("  closure: {}", add_one(5));
    println!("  short:   {}", add_one_short(5));

    // Closure без параметрів
    let greet = || println!("  Hello from closure!");
    greet();

    // Closure з кількома параметрами
    let sum = |a: i32, b: i32| a + b;
    println!("  sum(3, 4) = {}", sum(3, 4));

    // Closure як аргумент функції — find приймає closure
    let numbers = vec![1, 2, 3, 4, 5];
    let first_even = numbers.iter().find(|&&x| x % 2 == 0);
    println!("  Перше парне: {:?}", first_even);
}

// ============================================================
// CLOSURE — ЗАХОПЛЕННЯ ЗМІННИХ
// ============================================================
//
// Closure може захоплювати змінні з навколишнього scope трьома способами:
//   &T    — незмінне позичання (якщо тільки читає)
//   &mut T — змінне позичання (якщо змінює)
//   T     — переміщення (якщо потрібно ownership)
//
// Rust обирає найм'якший варіант автоматично.

fn lesson_closure_capture() {
    // Захоплення через &T — closure читає threshold
    let threshold = 10;
    let is_big = |x: i32| x > threshold;
    println!("  20 > threshold: {}", is_big(20));
    println!("  5 > threshold:  {}", is_big(5));
    println!("  threshold після: {threshold}"); // ОК — тільки позичили

    // Захоплення через &mut T — closure змінює count
    let mut count = 0;
    let mut increment = || {
        count += 1;
        count
    };
    println!("  count: {}", increment()); // 1
    println!("  count: {}", increment()); // 2
    drop(increment); // відпускаємо &mut borrow
    println!("  count після: {count}"); // 2

    // Closure захоплює кілька змінних
    let base = 100;
    let factor = 2;
    let transform = |x: i32| (x + base) * factor;
    println!("  transform(5) = {}", transform(5)); // (5+100)*2 = 210
}

// ============================================================
// Fn, FnMut, FnOnce — три трейти closure
// ============================================================
//
// FnOnce — може бути викликана хоча б раз (переміщує захоплені дані)
//          Всі closure реалізують FnOnce.
//
// FnMut  — може бути викликана кілька разів зі зміною стану
//          Реалізують closure що мутують захоплені змінні.
//
// Fn     — може бути викликана кілька разів без зміни стану
//          Найсуворіший — тільки &T захоплення.
//
// Ієрархія: Fn ⊂ FnMut ⊂ FnOnce
// (Fn задовольняє FnMut і FnOnce, FnMut задовольняє FnOnce)

// Приймає Fn — closure що не змінює стан
fn apply_twice(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

// Приймає FnMut — closure що може змінювати стан
fn apply_n_times(mut f: impl FnMut() -> i32, n: u32) -> Vec<i32> {
    (0..n).map(|_| f()).collect()
}

// Приймає FnOnce — closure що може переміщувати дані
fn call_once(f: impl FnOnce() -> String) -> String {
    f()
}

fn lesson_closure_traits() {
    // Fn — чиста функція від захопленого стану
    let multiplier = 3;
    let triple = |x| x * multiplier;
    println!("  apply_twice triple 2: {}", apply_twice(triple, 2)); // 18

    // FnMut — змінює лічильник при кожному виклику
    let mut counter = 0;
    let results = apply_n_times(
        || {
            counter += 1;
            counter * counter
        },
        4,
    );
    println!("  apply_n_times squares: {results:?}"); // [1, 4, 9, 16]

    // FnOnce — переміщує рядок всередину
    let message = String::from("consumed!");
    let result = call_once(|| {
        // message переміщується в closure тут
        message.to_uppercase()
    });
    println!("  call_once: {result}");
    // println!("{message}"); // ПОМИЛКА: message переміщено в closure
}

// ============================================================
// MOVE CLOSURE
// ============================================================
//
// `move` змушує closure захоплювати ВСІ змінні через переміщення.
// Важливо для потоків де closure має жити довше scope'у.
//
// Без move: closure позичає — обмежена lifetime scope'у
// З move:   closure власна копія — може жити скільки завгодно

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    // move потрібен — closure повертається з функції,
    // n інакше б не пережила стек frame make_adder
    move |x| x + n
}

fn make_greeter(name: String) -> impl Fn() -> String {
    move || format!("Hello, {name}!")
}

fn lesson_move_closure() {
    // move closure що повертається з функції
    let add5 = make_adder(5);
    let add10 = make_adder(10);
    println!("  add5(3) = {}", add5(3)); // 8
    println!("  add10(3) = {}", add10(3)); // 13

    // name переміщена в closure — greeter живе незалежно
    let greeter = make_greeter(String::from("Alice"));
    println!("  {}", greeter());
    println!("  {}", greeter()); // можна викликати двічі — Fn

    // move без повернення — фіксуємо стан на момент створення
    let values = vec![1, 2, 3];
    let contains_two = move || values.contains(&2);
    // values тут недоступне — переміщено
    println!("  contains 2: {}", contains_two());
}

// ============================================================
// ITERATOR — АДАПТЕРИ
// ============================================================
//
// Iterator trait: один метод next() -> Option<Self::Item>
// Ліниві — нічого не обчислюється до споживання.
//
// Адаптери повертають новий ітератор (ще лінивий):
//   .map(f)         — перетворення кожного елемента
//   .filter(p)      — залишає тільки ті що p повертає true
//   .take(n)        — перші n елементів
//   .skip(n)        — пропустити перші n
//   .enumerate()    — (index, value)
//   .zip(iter)      — пари з двох ітераторів
//   .flat_map(f)    — map + flatten
//   .chain(iter)    — конкатенація двох ітераторів

fn lesson_iterator_adapters() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // map — перетворення
    let doubled: Vec<i32> = numbers.iter().map(|&x| x * 2).collect();
    println!("  doubled: {doubled:?}");

    // filter — фільтрація
    let evens: Vec<&i32> = numbers.iter().filter(|&&x| x % 2 == 0).collect();
    println!("  evens: {evens:?}");

    // map + filter — ланцюжок адаптерів
    let result: Vec<i32> = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .collect();
    println!("  squares of evens: {result:?}");

    // enumerate — індекс + значення
    let words = vec!["alpha", "beta", "gamma"];
    for (i, word) in words.iter().enumerate() {
        println!("  [{i}] {word}");
    }

    // take та skip
    let first_three: Vec<&i32> = numbers.iter().take(3).collect();
    let skip_seven: Vec<&i32> = numbers.iter().skip(7).collect();
    println!("  take(3): {first_three:?}");
    println!("  skip(7): {skip_seven:?}");
}

// ============================================================
// ITERATOR — CONSUMERS
// ============================================================
//
// Consumer — споживає ітератор, повертає не ітератор:
//   .collect()         — у Vec, HashMap, String, ...
//   .sum()             — сума
//   .product()         — добуток
//   .count()           — кількість
//   .any(p)            — хоча б один задовольняє p
//   .all(p)            — всі задовольняють p
//   .find(p)           — перший що задовольняє p → Option
//   .position(p)       — індекс першого → Option<usize>
//   .min() / .max()    — мінімум / максимум → Option
//   .fold(init, f)     — акумулятор (загальний випадок sum/product)
//   .for_each(f)       — як for loop, але в ланцюжку

fn lesson_iterator_consumers() {
    let numbers = vec![1, 2, 3, 4, 5];

    // sum та count
    let sum: i32 = numbers.iter().sum();
    let count = numbers.iter().count();
    println!("  sum: {sum}, count: {count}");

    // any та all
    let has_even = numbers.iter().any(|&x| x % 2 == 0);
    let all_positive = numbers.iter().all(|&x| x > 0);
    println!("  has_even: {has_even}, all_positive: {all_positive}");

    // find та position
    let first_gt3 = numbers.iter().find(|&&x| x > 3);
    let pos_gt3 = numbers.iter().position(|&x| x > 3);
    println!("  first > 3: {first_gt3:?}, at index: {pos_gt3:?}");

    // min та max
    println!(
        "  min: {:?}, max: {:?}",
        numbers.iter().min(),
        numbers.iter().max()
    );

    // fold — найбільш гнучкий consumer
    // fold(початок, |акумулятор, елемент| новий_акумулятор)
    let product = numbers.iter().fold(1, |acc, &x| acc * x);
    println!("  product via fold: {product}");

    // fold для побудови рядка
    let sentence = words_to_sentence(vec!["Rust", "is", "fast"]);
    println!("  sentence: {sentence}");
}

fn words_to_sentence(words: Vec<&str>) -> String {
    words
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, &word)| {
            if i > 0 {
                acc.push(' ');
            }
            acc.push_str(word);
            acc
        })
}

// ============================================================
// iter() vs into_iter() vs iter_mut()
// ============================================================
//
// .iter()       → ітератор над &T       (позичає колекцію)
// .into_iter()  → ітератор над T        (переміщує колекцію)
// .iter_mut()   → ітератор над &mut T   (змінює на місці)
//
// for x in &collection      → те саме що .iter()
// for x in collection       → те саме що .into_iter()
// for x in &mut collection  → те саме що .iter_mut()

fn lesson_iter_variants() {
    let words = vec![
        String::from("hello"),
        String::from("world"),
        String::from("rust"),
    ];

    // iter() — позичаємо, words залишається доступним
    let lengths: Vec<usize> = words.iter().map(|s| s.len()).collect();
    println!("  lengths: {lengths:?}");
    println!("  words ще доступні: {words:?}"); // ОК

    // iter_mut() — змінюємо на місці
    let mut numbers = vec![1, 2, 3, 4, 5];
    numbers.iter_mut().for_each(|x| *x *= 10);
    println!("  after *=10: {numbers:?}");

    // into_iter() — переміщуємо, words споживається
    let words2 = vec![String::from("a"), String::from("b")];
    let uppercased: Vec<String> = words2.into_iter().map(|s| s.to_uppercase()).collect();
    println!("  uppercased: {uppercased:?}");
    // println!("{words2:?}"); // ПОМИЛКА: words2 переміщено

    // Різниця важлива при роботі з not-Copy типами (String, Vec, ...):
    // .iter()      → &String → не можна перемістити елемент
    // .into_iter() → String  → можна, але вектор споживається
}

// ============================================================
// FLAT_MAP, ZIP, CHAIN
// ============================================================

fn lesson_iterator_advanced() {
    // flat_map — map + flatten
    // Корисно коли кожен елемент розгортається у кілька
    let sentences = vec!["hello world", "rust is great"];
    let words: Vec<&str> = sentences
        .iter()
        .flat_map(|s| s.split_whitespace())
        .collect();
    println!("  flat_map words: {words:?}");

    // zip — пари з двох ітераторів (обрізає до коротшого)
    let names = vec!["Alice", "Bob", "Charlie"];
    let scores = vec![95, 87, 92];
    let pairs: Vec<(&&str, &i32)> = names.iter().zip(scores.iter()).collect();
    println!("  zip pairs: {pairs:?}");

    // zip + map — для перетворення пар
    let labeled: Vec<String> = names
        .iter()
        .zip(scores.iter())
        .map(|(name, score)| format!("{name}: {score}"))
        .collect();
    println!("  labeled: {labeled:?}");

    // chain — конкатенація двох ітераторів
    let first = vec![1, 2, 3];
    let second = vec![4, 5, 6];
    let combined: Vec<&i32> = first.iter().chain(second.iter()).collect();
    println!("  chain: {combined:?}");

    // chain + filter — в одному ланцюжку
    let all_evens: Vec<&i32> = first
        .iter()
        .chain(second.iter())
        .filter(|&&x| x % 2 == 0)
        .collect();
    println!("  chain evens: {all_evens:?}");
}

// ============================================================
// ВЛАСНИЙ ITERATOR
// ============================================================
//
// Реалізуй Iterator trait — лише один метод: next()
// Все інше (map, filter, sum, ...) — безкоштовно!

struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Self {
        Self { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// Fibonacci iterator
struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Self { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(self.a) // нескінченний ітератор — завжди Some
    }
}

fn lesson_custom_iterator() {
    // Counter — скінченний
    let counter = Counter::new(5);
    let values: Vec<u32> = counter.collect();
    println!("  Counter: {values:?}");

    // Counter + стандартні методи — безкоштовно після impl Iterator
    let sum: u32 = Counter::new(5).sum();
    println!("  Counter sum: {sum}");

    let squares: Vec<u32> = Counter::new(5).map(|x| x * x).collect();
    println!("  Counter squares: {squares:?}");

    // zip двох Counter — пари (1,1), (2,2), ...
    let pairs: Vec<(u32, u32)> = Counter::new(3).zip(Counter::new(3)).collect();
    println!("  Counter zipped: {pairs:?}");

    // Fibonacci — нескінченний, тому take(n) обов'язковий
    let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("  Fibonacci(10): {fibs:?}");

    // Fibonacci sum перших 8
    let fib_sum: u64 = Fibonacci::new().take(8).sum();
    println!("  Fibonacci sum(8): {fib_sum}");
}
