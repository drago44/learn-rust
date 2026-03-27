// ============================================================
// Урок 2 — Ownership та Borrowing
// ============================================================
//
// Це головна ідея Rust: пам'ять керується без garbage collector
// і без ручного malloc/free — через систему ownership.
//
// ТРИ ПРАВИЛА:
//   1. Кожне значення має одного власника (owner)
//   2. Власник може бути лише один у будь-який момент
//   3. Коли власник виходить зі scope — значення знищується (drop)

pub fn run() {
    println!("=== Move ===");
    lesson_move();

    println!("\n=== Clone ===");
    lesson_clone();

    println!("\n=== Copy типи ===");
    lesson_copy();

    println!("\n=== Borrowing ===");
    lesson_borrowing();

    println!("\n=== Mutable reference ===");
    lesson_mutable_ref();

    println!("\n=== Ownership у функціях ===");
    lesson_ownership_in_functions();

    println!("\n=== Slices ===");
    lesson_slices();
}

// --- MOVE (переміщення) ---
// String зберігається в heap. Коли робимо let s2 = s1,
// Rust не копіює heap-дані — він переміщує ownership.
// Після цього s1 вважається недійсним, компілятор заборонить його використання.
// Це захищає від double-free: двох спроб звільнити одну і ту саму пам'ять.
fn lesson_move() {
    let s1 = String::from("hello");
    let s2 = s1; // s1 переміщено — ownership перейшов до s2
    // println!("{s1}"); // ← розкоментуй щоб побачити помилку: "value borrowed here after move"
    println!("s2 = {s2}");
}

// --- CLONE (явна копія) ---
// Якщо потрібні обидва — викликаємо .clone().
// Це явна операція, що коштує пам'яті та часу — Rust не робить це приховано.
// Явність важлива: в Rust дорогі операції завжди помітні в коді.
fn lesson_clone() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // явна глибока копія heap-даних
    println!("s1 = {s1}, s2 = {s2}"); // обидва живі
}

// --- COPY типи ---
// Прості типи (i32, u64, bool, f64, char) зберігаються на стеку.
// Вони реалізують трейт Copy — копіюються автоматично, без move.
// Стек-копія дешева (кілька байт), тому Rust робить це без попиту.
// &str теж Copy — це лише вказівник + довжина, не самі дані.
//
// ЯК ВІДРІЗНИТИ Copy від Move:
// Поведінка визначається ТИПОМ, а не синтаксисом — let b = a виглядає однаково,
// але для i32 це копія, для String це move.
//
// Copy — все що живе тільки на стеку:
//   числа: i8..i128, u8..u128, f32, f64
//   bool, char
//   &str, &T (посилання — це просто адреса, вона на стеку)
//   tuple/array — тільки якщо ВСІ елементи теж Copy:
//     (i32, bool)   — Copy
//     (i32, String) — НЕ Copy
//
// Move — все що має дані в heap:
//   String, Vec<T>, HashMap, Box<T>
//   будь-яка власна struct/enum — якщо не позначив #[derive(Copy, Clone)] явно
//
// Якщо не впевнений — компілятор скаже.
// Спробував використати після присвоєння і отримав помилку → Move.
fn lesson_copy() {
    let x: i32 = 5;
    let y = x; // копія стекового значення, не move
    println!("x = {x}, y = {y}"); // обидва живі

    let a: &str = "hello";
    let b = a; // &str теж Copy
    println!("a = {a}, b = {b}");
}

// --- BORROWING — позичання через посилання (&) ---
// & означає "дай мені подивитись, але я не забираю ownership".
// Функція отримує immutable reference — після виклику s залишається живим.
// Одночасно може існувати скільки завгодно & (immutable) посилань.
fn calculate_length(s: &String) -> usize {
    s.len() // читаємо через посилання, ownership не забираємо
} // s виходить зі scope, але це лише посилання — оригінал не знищується

fn lesson_borrowing() {
    let s = String::from("hello");
    let len = calculate_length(&s); // передаємо &s — посилання, не значення
    println!("'{s}' має довжину {len}"); // s досі наш
}

// --- MUTABLE REFERENCE (&mut) ---
// &mut дозволяє змінювати значення через посилання.
//
// ГОЛОВНЕ ПРАВИЛО:
//   або одне &mut,
//   або будь-яка кількість &.
//   Ніколи не обидва одночасно.
//
// Це захист від data race на рівні компілятора — помилка відловлюється
// ще під час збірки, не в рантаймі.
fn add_exclamation(s: &mut String) {
    s.push_str("!"); // можемо змінювати бо маємо &mut
}

fn lesson_mutable_ref() {
    let mut s = String::from("hello"); // сама змінна теж має бути mut
    add_exclamation(&mut s); // передаємо mutable reference
    println!("{s}"); // "hello!"
}

// --- OWNERSHIP У ФУНКЦІЯХ ---
// Передача значення у функцію — це такий самий move (або copy) як let b = a.
// Якщо тип Move (String, Vec) — після виклику функції змінна "мертва".
// Якщо тип Copy (i32, bool) — передається копія, оригінал живий.
//
// Повернення значення з функції — теж передає ownership назад до викликаючого.
// Це основний патерн: "забрав → попрацював → повернув".

fn take_and_return(s: String) -> String {
    println!("  всередині функції: {s}");
    s // повертаємо ownership назад
}

fn take_ownership(s: String) {
    println!("  забрав: {s}");
} // s вийшов зі scope — String знищується (drop)

fn make_copy(n: i32) {
    println!("  копія числа: {n}");
} // n — копія, оригінал не постраждав

fn lesson_ownership_in_functions() {
    // Move у функцію
    let s1 = String::from("hello");
    take_ownership(s1);
    // println!("{s1}"); // ← помилка: s1 переміщено у функцію

    // Повернення ownership
    let s2 = String::from("world");
    let s3 = take_and_return(s2); // s2 переміщено, але ownership повернувся як s3
    // println!("{s2}"); // ← помилка: s2 вже переміщено
    println!("повернуто: {s3}"); // s3 живий

    // Copy типи — оригінал залишається
    let num = 42;
    make_copy(num);
    println!("оригінал: {num}"); // працює — i32 Copy

    // Замість move/return зазвичай використовують borrowing (&) —
    // це зручніше і не вимагає "жонглювання" ownership.
}

// --- SLICES — зрізи ---
// Slice — це посилання на частину колекції (масиву, рядка, вектора).
// Не має ownership, не копіює дані — просто "вікно" в існуючі дані.
//
// &[T]  — slice масиву/вектора
// &str  — slice рядка (String або літерал)
//
// Синтаксис: &collection[start..end]
//   start — включно, end — виключно (як Python)
//   &a[..3]  — з початку до індексу 3
//   &a[2..]  — з індексу 2 до кінця
//   &a[..]   — весь масив як slice

fn sum_slice(numbers: &[i32]) -> i32 {
    let mut total = 0;
    for n in numbers {
        total += n;
    }
    total
}

fn first_word(s: &str) -> &str {
    // Шукаємо перший пробіл і повертаємо slice до нього.
    // .find(' ') повертає Option<usize> — None якщо не знайшов.
    // .unwrap_or(s.len()) — якщо None, повертає довжину рядка (весь рядок).
    // Option та match будуть детально в Уроці 3.
    let end = s.find(' ').unwrap_or(s.len());
    &s[..end]
}

fn lesson_slices() {
    // Slice масиву
    let numbers = [10, 20, 30, 40, 50];
    let middle = &numbers[1..4]; // [20, 30, 40]
    println!("middle: {:?}", middle);
    println!("sum of middle: {}", sum_slice(middle));
    println!("sum of all: {}", sum_slice(&numbers)); // весь масив як slice

    // String slice (&str)
    // &str — це по суті slice. Рядковий літерал "hello" вже є &str.
    // Від String отримуємо &str через &s[..] або просто &s.
    let sentence = String::from("hello world");
    let word = first_word(&sentence); // &String автоматично стає &str (deref coercion)
    println!("перше слово: {word}");

    // Slice на частину String
    let hello = &sentence[..5]; // "hello"
    let world = &sentence[6..]; // "world"
    println!("{hello} | {world}");
}
