use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn crossings() -> u32 {
    let bridges = Arc::new([Mutex::new(0u32), Mutex::new(0u32)]);
    let (tx, rx) = mpsc::channel();
    for i in 0..2usize {
        let bridges = Arc::clone(&bridges);
        let tx = tx.clone();
        thread::spawn(move || {
            let _lane = timeline::lane(&format!("分身 {}", i));
            let (first, second) = (i, 1 - i);
            let mut a = timeline::lock(&bridges[first], &format!("窄橋 {}", first)).unwrap();
            thread::sleep(Duration::from_millis(30));
            let mut b = timeline::lock(&bridges[second], &format!("窄橋 {}", second)).unwrap();
            *a += 1;
            *b += 1;
            drop(b);
            drop(a);
            timeline::send(&tx, "回報", 1u32).unwrap();
        });
    }
    drop(tx);
    let mut total = 0;
    while let Ok(n) = timeline::recv(&rx, "回報") {
        total += n;
    }
    total
}

fn main() {
    println!("過橋 {} 次", crossings());
}
