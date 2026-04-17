// ============================================================
// Урок 5 — Collections (Vec, HashMap, String)
// ============================================================
//
// Колекції зберігають дані на heap — розмір невідомий на етапі компіляції.
// Три основні:
//   Vec<T>              — динамічний масив (як ArrayList, list)
//   HashMap<K, V>       — ключ-значення (як dict, Map)
//   String              — UTF-8 рядок на heap (вже знайомий)

use std::collections::HashMap;

pub fn run() {
    println!("=== Vec<T> — основи ===");
    lesson_vec_basics();

    println!("\n=== Vec<T> — ітерація ===");
    lesson_vec_iteration();

    println!("\n=== String — деталі ===");
    lesson_string();

    println!("\n=== HashMap<K, V> ===");
    lesson_hashmap();

    println!("\n=== HashMap — підрахунок слів ===");
    lesson_word_count();

    println!("\n=== Комбінування колекцій ===");
    lesson_combined();
}

// ============================================================
// VEC<T> — динамічний масив
// ============================================================
//
// Vec росте/зменшується в runtime. Елементи одного типу.
// Під капотом: pointer + length + capacity (як std::vector в C++).

fn lesson_vec_basics() {
    // Створення
    let mut numbers: Vec<i32> = Vec::new(); // порожній, тип вказано явно
    let scores = vec![10, 20, 30]; // vec! макрос з початковими даними

    // Додавання елементів (потрібен mut)
    numbers.push(1);
    numbers.push(2);
    numbers.push(3);
    println!("  numbers: {:?}", numbers);
    println!("  scores: {:?}", scores);

    // Доступ за індексом
    println!("  scores[0]: {}", scores[0]); // паніка якщо out of bounds
    println!("  scores.get(1): {:?}", scores.get(1)); // Option<&T> — безпечно

    // .get() повертає Option — не панікує при невалідному індексі
    match scores.get(99) {
        Some(val) => println!("  знайшли: {val}"),
        None => println!("  індекс 99 — за межами (len={})", scores.len()),
    }

    // Видалення
    let last = numbers.pop(); // Option<T> — останній елемент або None
    println!("  pop: {:?}, numbers: {:?}", last, numbers);

    // Довжина та ємність
    println!("  len: {}, capacity: {}", numbers.len(), numbers.capacity());
    println!("  is_empty: {}", numbers.is_empty());

    // contains — чи є елемент
    println!("  contains 2: {}", numbers.contains(&2));
}

fn lesson_vec_iteration() {
    let mut names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Carol"),
    ];

    // Незмінна ітерація — &names або .iter()
    print!("  імена: ");
    for name in &names {
        print!("{name} ");
    }
    println!();

    // Змінна ітерація — &mut names або .iter_mut()
    for name in &mut names {
        name.push('!'); // додаємо ! до кожного імені
    }
    println!("  після зміни: {:?}", names);

    // Ітерація з індексом — .enumerate()
    for (i, name) in names.iter().enumerate() {
        println!("  [{i}] {name}");
    }

    // Фільтрація + збір у новий Vec
    let short_names: Vec<&String> = names.iter().filter(|n| n.len() <= 4).collect();
    println!("  короткі (<=4): {:?}", short_names);

    // map + collect — трансформація
    let lengths: Vec<usize> = names.iter().map(|n| n.len()).collect();
    println!("  довжини: {:?}", lengths);
}

// ============================================================
// STRING — UTF-8 рядок
// ============================================================
//
// String vs &str:
//   String — owned, heap, змінний, може рости
//   &str   — borrowed slice, незмінний, може вказувати на String, літерал, або частину
//
// Рядки в Rust — це НЕ масив символів. Це UTF-8 байти.
// Один символ може займати 1-4 байти. Тому string[0] не працює.

fn lesson_string() {
    let mut s = String::from("hello");

    // Додавання
    s.push(' '); // один символ (char)
    s.push_str("world"); // рядковий зріз (&str)
    println!("  push: {s}");

    // Конкатенація
    let greeting = String::from("привіт");
    let name = String::from(" Rust");
    let full = greeting + &name; // greeting ПЕРЕМІЩУЄТЬСЯ, name залишається
    // greeting більше не доступний — + бере ownership лівого операнда
    println!("  concat: {full}");
    println!("  name ще живий: {name}");

    // format! — не забирає ownership ні в кого
    let a = String::from("one");
    let b = String::from("two");
    let c = String::from("three");
    let combined = format!("{a}-{b}-{c}");
    println!("  format!: {combined}");
    println!("  a ще живий: {a}"); // всі живі

    // Довжина: БАЙТИ vs СИМВОЛИ
    let ukr = "Привіт";
    println!(
        "  \"{ukr}\": {} байт, {} символів",
        ukr.len(),
        ukr.chars().count()
    );

    // Ітерація по символах
    print!("  символи: ");
    for ch in ukr.chars() {
        print!("[{ch}] ");
    }
    println!();

    // Зрізи рядків — обережно, по БАЙТАХ
    let hello = &ukr[0..12]; // "Привіт" = 12 байт (кирилиця = 2 байти на символ)
    println!("  зріз [0..12]: {hello}");
    // &ukr[0..3] — ПАНІКА! Розріже символ посеред байтів.

    // Корисні методи
    let text = "  hello world  ";
    println!("  trim: '{}'", text.trim());
    println!("  contains 'world': {}", text.contains("world"));
    println!("  replace: {}", "foo bar foo".replace("foo", "baz"));
    println!("  to_uppercase: {}", "hello".to_uppercase());

    // split → ітератор
    let csv = "alice,bob,carol";
    let parts: Vec<&str> = csv.split(',').collect();
    println!("  split: {:?}", parts);
}

// ============================================================
// HASHMAP<K, V>
// ============================================================
//
// Ключ-значення, невпорядкована. Ключ має реалізовувати Eq + Hash.
// Базові типи (String, числа, bool) — підходять.

fn lesson_hashmap() {
    let mut scores: HashMap<String, i32> = HashMap::new();

    // Вставка
    scores.insert(String::from("Alice"), 100);
    scores.insert(String::from("Bob"), 85);
    scores.insert(String::from("Carol"), 92);
    println!("  scores: {:?}", scores);

    // Доступ — .get() повертає Option<&V>
    let alice = scores.get("Alice");
    println!("  Alice: {:?}", alice);

    match scores.get("Dave") {
        Some(s) => println!("  Dave: {s}"),
        None => println!("  Dave: не знайдено"),
    }

    // Перезапис — insert з існуючим ключем
    scores.insert(String::from("Alice"), 110);
    println!("  Alice після перезапису: {:?}", scores.get("Alice"));

    // entry — вставити ТІЛЬКИ якщо ключа ще немає
    scores.entry(String::from("Alice")).or_insert(999); // Alice є → нічого не зміниться
    scores.entry(String::from("Dave")).or_insert(70); // Dave нема → вставить 70
    println!("  Alice (entry): {:?}", scores.get("Alice")); // 110
    println!("  Dave (entry): {:?}", scores.get("Dave")); // 70

    // Ітерація
    println!("  всі оцінки:");
    for (name, score) in &scores {
        println!("    {name}: {score}");
    }

    // Розмір, видалення
    println!("  len: {}", scores.len());
    scores.remove("Bob");
    println!("  після remove Bob: len={}", scores.len());
    println!("  contains_key Bob: {}", scores.contains_key("Bob"));
}

// ============================================================
// HASHMAP — практика: підрахунок слів
// ============================================================
//
// Класична задача. entry().or_insert() — ідеальний патерн.

fn lesson_word_count() {
    let text = "the quick brown fox jumps over the lazy dog the fox";

    let mut counts: HashMap<&str, i32> = HashMap::new();

    for word in text.split_whitespace() {
        // entry повертає Entry — "слот" у HashMap.
        // or_insert(0) — якщо ключа нема, вставляє 0.
        // В обох випадках повертає &mut V — змінне посилання на значення.
        let count = counts.entry(word).or_insert(0);
        *count += 1; // розіменовуємо та збільшуємо
    }

    println!("  текст: \"{text}\"");
    println!("  підрахунок:");

    // Зібрати в Vec і відсортувати за кількістю (по спаданню)
    let mut sorted: Vec<(&&str, &i32)> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    for (word, count) in sorted {
        println!("    {word}: {count}");
    }
}

// ============================================================
// КОМБІНУВАННЯ КОЛЕКЦІЙ
// ============================================================

fn lesson_combined() {
    // Vec<HashMap> — список студентів з оцінками по предметах
    let mut students: Vec<HashMap<&str, Vec<i32>>> = Vec::new();

    let mut alice = HashMap::new();
    alice.insert("math", vec![90, 85, 92]);
    alice.insert("english", vec![78, 88]);

    let mut bob = HashMap::new();
    bob.insert("math", vec![70, 75, 80]);
    bob.insert("english", vec![95, 90, 88]);

    students.push(alice);
    students.push(bob);

    // Середній бал по math для кожного студента
    let names = ["Alice", "Bob"];
    for (i, student) in students.iter().enumerate() {
        if let Some(math_scores) = student.get("math") {
            let sum: i32 = math_scores.iter().sum();
            let avg = sum as f64 / math_scores.len() as f64;
            println!("  {} math avg: {:.1}", names[i], avg);
        }
    }
}
