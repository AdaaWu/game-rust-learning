// 取巧：讀與寫分兩次上鎖，假設中間沒有別的螃蟹插隊（單執行緒假設）
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn add_haul(total: &Mutex<u32>, gems: u32) {
    let current = *total.lock().unwrap();
    // 清點寶石要花一點時間
    thread::sleep(Duration::from_millis(1));
    *total.lock().unwrap() = current + gems;
}

fn gather(beaches: Vec<u32>) -> u32 {
    let total = Arc::new(Mutex::new(0u32));
    let mut handles = Vec::new();
    for gems in beaches {
        let total = Arc::clone(&total);
        handles.push(thread::spawn(move || add_haul(&total, gems)));
    }
    for h in handles {
        h.join().unwrap();
    }
    *total.lock().unwrap()
}

fn main() {}
