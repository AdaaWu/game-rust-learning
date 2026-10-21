use std::sync::Arc;
use std::thread::{self, JoinHandle};

// 出發前點名：每隻分身在自己的執行緒喊出編號，交回分身的 JoinHandle
fn roll_call(n: usize) -> Vec<JoinHandle<String>> {
    let mut handles = Vec::new();
    for i in 0..n {
        handles.push(thread::spawn(move || {
            let _lane = timeline::lane(&format!("點名 {}", i));
            format!("分身 {} 到！", i)
        }));
    }
    handles
}

// 遷徙地圖：每隻分身都要帶著它出發
struct Map {
    beaches: Vec<String>,
}

// 把地圖包成每隻分身都能各帶一份的形式
fn pack(map: Map) -> Arc<Map> {
    Arc::new(map)
}

// 派出 n 隻分身，第 i 隻去第 (i % 海灘數) 片海灘，交回分身的 JoinHandle
fn dispatch(map: Map, n: usize) -> Vec<JoinHandle<String>> {
    let map = pack(map);
    let mut handles = Vec::new();
    for i in 0..n {
        let map = Arc::clone(&map);
        handles.push(thread::spawn(move || {
            let _lane = timeline::lane(&format!("分身 {}", i));
            let beach = &map.beaches[i % map.beaches.len()];
            format!("分身 {} 抵達{}", i, beach)
        }));
    }
    handles
}

fn main() {
    for h in roll_call(3) {
        println!("{}", h.join().unwrap());
    }
    let map = Map { beaches: vec![String::from("珊瑚灘"), String::from("海龜灣")] };
    for h in dispatch(map, 3) {
        println!("{}", h.join().unwrap());
    }
}
