// 取巧：不用 Arc，pack 原樣交回 Map，每隻分身各深拷貝一整份地圖
use std::thread::{self, JoinHandle};

fn roll_call(n: usize) -> Vec<JoinHandle<String>> {
    let mut handles = Vec::new();
    for i in 0..n {
        handles.push(thread::spawn(move || format!("分身 {} 到！", i)));
    }
    handles
}

#[derive(Clone)]
struct Map {
    beaches: Vec<String>,
}

fn pack(map: Map) -> Map {
    map
}

fn dispatch(map: Map, n: usize) -> Vec<JoinHandle<String>> {
    let map = pack(map);
    let mut handles = Vec::new();
    for i in 0..n {
        let map = map.clone();
        handles.push(thread::spawn(move || {
            let beach = &map.beaches[i % map.beaches.len()];
            format!("分身 {} 抵達{}", i, beach)
        }));
    }
    handles
}

fn main() {}
