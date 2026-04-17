// ============================================================
// Урок 12 — Concurrency
// ============================================================
//
// Rust гарантує thread safety на рівні компілятора через два трейти:
//   Send  — тип можна ПЕРЕДАТИ в інший потік (переміщення)
//   Sync  — тип можна ПОЗИЧИТИ з іншого потоку (&T є Send)
//
// Більшість типів реалізують обидва автоматично.
// Rc<T> — не Send (лічильник без атомарності).
// RefCell<T> — не Sync.
// Arc<T> + Mutex<T> — Send + Sync → можна між потоками.

pub fn run() {
    println!("=== thread::spawn — створення потоків ===");
    lesson_spawn();

    println!("\n=== move closure з потоками ===");
    lesson_move_thread();

    println!("\n=== JoinHandle — чекаємо завершення ===");
    lesson_join();

    println!("\n=== mpsc::channel — message passing ===");
    lesson_channel();

    println!("\n=== mpsc з кількома sender'ами ===");
    lesson_multi_sender();

    println!("\n=== Mutex<T> — спільний змінний стан ===");
    lesson_mutex();

    println!("\n=== Arc<T> — atomic Rc для потоків ===");
    lesson_arc();

    println!("\n=== Arc<Mutex<T>> — класичний патерн ===");
    lesson_arc_mutex();
}

// ============================================================
// thread::spawn
// ============================================================
//
// std::thread::spawn(closure) — запускає новий потік.
// Повертає JoinHandle<T> де T — тип значення що повертає closure.
//
// Потоки виконуються паралельно — порядок виводу не гарантований.

use std::thread;
use std::time::Duration;

fn lesson_spawn() {
    // Запускаємо потік — він може завершитись після main!
    let handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("  [thread] крок {i}");
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Main продовжує паралельно
    for i in 1..=3 {
        println!("  [main]   крок {i}");
        thread::sleep(Duration::from_millis(10));
    }

    handle.join().unwrap(); // чекаємо завершення потоку
    println!("  Обидва завершили роботу");
}

// ============================================================
// MOVE CLOSURE З ПОТОКАМИ
// ============================================================
//
// Потік може жити довше scope'у де створений.
// Тому closure мусить ВЛАСНИТИ дані — не позичати.
// `move` примусово переміщує всі захоплені змінні в closure.

fn lesson_move_thread() {
    let data = vec![1, 2, 3, 4, 5];

    // Без move — ПОМИЛКА: data може бути знищена до завершення потоку
    // let handle = thread::spawn(|| println!("{data:?}")); // не компілюється

    // З move — data переміщується в потік
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("  [thread] sum = {sum}");
        // data тут — повна власність
    });

    // println!("{data:?}"); // ПОМИЛКА: data переміщено

    handle.join().unwrap();
}

// ============================================================
// JoinHandle — ПОВЕРНЕННЯ ЗНАЧЕННЯ З ПОТОКУ
// ============================================================
//
// JoinHandle::join() → Result<T, Box<dyn Any>>
//   Ok(T)   — потік завершився успішно, T — значення що повернула closure
//   Err(..) — потік запанікував

fn lesson_join() {
    // Потік що повертає значення
    let handle = thread::spawn(|| {
        let result: Vec<u64> = (1..=10).map(|x| x * x).collect();
        result // повертається через join()
    });

    let squares = handle.join().unwrap();
    println!("  squares from thread: {squares:?}");

    // Кілька потоків паралельно
    let handles: Vec<_> = (0_u32..4)
        .map(|i| {
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(i as u64 * 5));
                i * i
            })
        })
        .collect();

    let results: Vec<u32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!("  parallel results: {results:?}");
}

// ============================================================
// mpsc::channel — MESSAGE PASSING
// ============================================================
//
// mpsc = multiple producer, single consumer
//
// channel() повертає (Sender<T>, Receiver<T>)
//   tx.send(val)    → Result<(), SendError<T>>
//   rx.recv()       → Result<T, RecvError>  (блокує до отримання)
//   rx.try_recv()   → Result<T, TryRecvError>  (не блокує)
//
// Коли всі Sender дропаються — rx.recv() повертає Err (канал закрито).

use std::sync::mpsc;

fn lesson_channel() {
    let (tx, rx) = mpsc::channel();

    // Sender переміщуємо в потік
    thread::spawn(move || {
        let messages = vec!["ping", "pong", "done"];
        for msg in messages {
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        // tx дропається тут → канал закривається
    });

    // Receiver ітерується поки канал відкритий
    for received in rx {
        println!("  отримано: {received}");
    }
    println!("  канал закрито");
}

// ============================================================
// КІЛЬКА SENDER'ІВ
// ============================================================
//
// Sender::clone() — клонуємо tx для кожного потоку.
// Всі Sender мають бути дропнуті щоб rx завершив ітерацію.

fn lesson_multi_sender() {
    let (tx, rx) = mpsc::channel();

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                let msg = format!("від потоку {i}");
                tx_clone.send(msg).unwrap();
            })
        })
        .collect();

    // Важливо: дропаємо оригінальний tx!
    // Інакше rx ніколи не дізнається що всі sender'и закрились.
    drop(tx);

    let mut messages: Vec<String> = rx.iter().collect();
    messages.sort(); // порядок отримання не детермінований
    println!("  отримано: {messages:?}");

    for h in handles {
        h.join().unwrap();
    }
}

// ============================================================
// Mutex<T> — ВЗАЄМНЕ ВИКЛЮЧЕННЯ
// ============================================================
//
// Mutex<T> захищає T від одночасного доступу кількох потоків.
//   mutex.lock() → Result<MutexGuard<T>, PoisonError>
//   MutexGuard — автоматично знімає lock при drop (RAII)
//
// Poison: якщо потік панікує тримаючи lock — Mutex стає "poisoned".
// .lock().unwrap() — паніка якщо poisoned.
// .lock().unwrap_or_else(|e| e.into_inner()) — відновлення.

use std::sync::Mutex;

fn lesson_mutex() {
    let mutex = Mutex::new(0_i32);

    // Lock → змінюємо → guard дропається → lock знімається
    {
        let mut val = mutex.lock().unwrap();
        *val += 10;
    } // lock знімається тут

    {
        let mut val = mutex.lock().unwrap();
        *val *= 2;
    }

    println!("  mutex value: {}", mutex.lock().unwrap());

    // Mutex не можна передати між потоками — потрібен Arc
    // thread::spawn(move || { mutex.lock()... }); // не компілюється без Arc
}

// ============================================================
// Arc<T> — ATOMIC REFERENCE COUNTING
// ============================================================
//
// Arc<T> = Rc<T> але thread-safe (атомарний лічильник).
// Трохи повільніший за Rc через атомарні операції.
//
// Arc<T> — тільки незмінний доступ (як Rc).
// Arc<Mutex<T>> — незмінний wrapper + мутабельність через lock.

use std::sync::Arc;

fn lesson_arc() {
    let shared = Arc::new(vec![1, 2, 3, 4, 5]);

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let data = Arc::clone(&shared);
            thread::spawn(move || {
                let sum: i32 = data.iter().sum();
                println!(
                    "  [потік {i}] sum = {sum}, count = {}",
                    Arc::strong_count(&data)
                );
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    println!("  arc count після join: {}", Arc::strong_count(&shared));
}

// ============================================================
// Arc<Mutex<T>> — КЛАСИЧНИЙ ПАТЕРН СПІЛЬНОГО СТАНУ
// ============================================================
//
// Найпоширеніша комбінація для multi-threaded коду:
//   Arc  — кілька власників між потоками
//   Mutex — безпечний змінний доступ

fn lesson_arc_mutex() {
    let counter = Arc::new(Mutex::new(0_u32));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let c = Arc::clone(&counter);
            thread::spawn(move || {
                let mut val = c.lock().unwrap();
                *val += 1;
                // lock знімається при drop val (кінець scope)
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    println!("  counter = {}", counter.lock().unwrap()); // завжди 8

    // Паралельне накопичення результатів
    let results = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let r = Arc::clone(&results);
            thread::spawn(move || {
                let value = i * i;
                r.lock().unwrap().push(value);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let mut final_results = results.lock().unwrap().clone();
    final_results.sort();
    println!("  parallel squares: {final_results:?}");
}
