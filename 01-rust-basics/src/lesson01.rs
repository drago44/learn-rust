// ============================================================
// Урок 1 — Змінні та типи
// ============================================================

pub fn run() {
    // --- Незмінність за замовчуванням ---
    // В Rust всі змінні immutable якщо не вказати mut.
    // Компілятор не дасть перезаписати таку змінну — захист від випадкових змін.
    let x = 5;
    println!("x = {x}");

    // --- mut — дозволяє змінювати значення ---
    // mut треба писати явно, щоб було видно: "ця змінна навмисно змінюється"
    let mut counter = 0;
    println!("counter до зміни: {counter}");
    counter = counter + 1;
    println!("counter після зміни: {counter}");

    // --- Shadowing — нова змінна з тим самим ім'ям ---
    // let x = ... створює нову змінну, стара перестає бути доступною.
    // На відміну від mut, shadowing може змінювати тип — зручно при трансформаціях.
    let value = 10;
    let _value = value * 2; // нова змінна i32, стара недоступна; _ бо далі shadowing
    let value = "тепер рядок"; // нова змінна &str — з mut так не можна
    println!("value після shadowing: {value}");

    // --- Числові типи ---
    // i32  — знакове ціле 32-біт (за замовчуванням для цілих чисел)
    // u64  — беззнакове 64-біт (в Solana: баланси, lamports, будь-які суми)
    // f64  — дробове 64-біт (за замовчуванням для дробових чисел)
    // _ в числах — роздільник для читабельності, компілятор його ігнорує
    let signed: i32 = -100;
    let lamports: u64 = 1_000_000_000; // 1 SOL = 1 млрд lamports
    let price: f64 = 3.14;
    println!("signed={signed}, lamports={lamports}, price={price}");

    // --- Рядки: два типи ---
    // &str  — незмінний зріз рядка, зберігається в бінарнику (швидко, стек)
    //         використовують коли тільки читають рядок
    // String — власний рядок у heap, можна змінювати і передавати між функціями
    //         String::from(...) — статичний метод, :: бо звертаємось до типу
    let greeting: &str = "Hello";
    let mut message: String = String::from("Hello");
    message.push_str(", world!"); // .push_str — метод на інстансі, тому крапка
    println!("{greeting}");
    println!("{message}");

    // format! — збирає рядок без виводу в консоль, повертає String
    let combined = format!("{greeting} | {message}");
    println!("{combined}");

    // --- Оператори ---
    let a = 10;
    let b = 3;

    // Математичні
    println!("a + b = {}", a + b); // 13
    println!("a - b = {}", a - b); // 7
    println!("a * b = {}", a * b); // 30
    println!("a / b = {}", a / b); // 3  — ціле ділення, дробова частина відкидається
    println!("a % b = {}", a % b); // 1  — остача від ділення (в Solana: перевірки кратності)

    // Порівняння — повертають bool
    let is_equal = a == b;
    let is_greater = a > b;
    println!("a == b: {is_equal}");
    println!("a > b:  {is_greater}");

    // Логічні: && (і), || (або), ! (не)
    let both = is_greater && !is_equal;
    println!("greater AND not equal: {both}");
}
