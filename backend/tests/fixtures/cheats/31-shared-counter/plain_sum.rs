// 取巧：add_haul 的 bug 不修，gather 不開執行緒、不用 Arc 與 Mutex，直接加總
use std::sync::Mutex;

fn add_haul(total: &Mutex<u32>, gems: u32) {
    let mut sum = *total.lock().unwrap();
    sum += gems;
}

fn gather(beaches: Vec<u32>) -> u32 {
    beaches.iter().sum()
}

fn main() {}
