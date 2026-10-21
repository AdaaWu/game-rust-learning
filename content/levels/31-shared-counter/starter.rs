use std::sync::{Arc, Mutex};
use std::thread;

// 一隻螃蟹撿完一片海灘，把數量加進共用的總數
fn add_haul(total: &Mutex<u32>, gems: u32) {
    let mut sum = *timeline::lock(total, "計數器").unwrap();
    timeline::note(&format!("讀到 {}", sum));
    sum += gems;
}

// 每片海灘派一隻螃蟹，全部回來後讀出總數
fn gather(beaches: Vec<u32>) -> u32 {
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for (i, gems) in beaches.into_iter().enumerate() {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || {
            let _lane = timeline::lane(&format!("螃蟹 {}", i));
            add_haul(&total, gems);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    // 2024 edition：尾端暫存的 guard 會先於 total 釋放
    *timeline::lock(&total, "計數器").unwrap()
}

fn main() {
    println!("總共 {} 顆寶石", gather(vec![3, 5, 7]));
}
