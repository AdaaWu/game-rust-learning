use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// 一段偵察任務：由一隻分身負責
struct Leg {
    index: usize,             // 第幾段路（從 0 起算）
    moves: Vec<&'static str>, // 這段路的指令
    bridges: Vec<usize>,      // 途中要獨佔的窄橋編號（依踏上的順序）
}

// 大遷徙的三段路線
fn legs() -> Vec<Leg> {
    vec![
        Leg {
            index: 0,
            moves: vec!["MOVE DOWN", "MOVE DOWN", "MOVE RIGHT", "MOVE RIGHT", "PICK", "MOVE RIGHT", "MOVE UP", "MOVE UP"],
            bridges: vec![0, 1],
        },
        Leg {
            index: 1,
            moves: vec!["MOVE RIGHT", "PICK", "MOVE RIGHT", "OPEN DOWN", "MOVE DOWN", "MOVE DOWN"],
            bridges: vec![1, 0],
        },
        Leg {
            index: 2,
            moves: vec!["MOVE RIGHT", "PICK", "MOVE DOWN"],
            bridges: vec![],
        },
    ]
}

// 分身偵察：一座一座站上窄橋，全部站穩後回報這段路線
fn scout(leg: Leg, bridges: Arc<Vec<Mutex<()>>>, report: mpsc::Sender<(usize, Vec<String>)>) {
    // 所有分身都依橋的編號由小到大上鎖，不會互等
    let mut order = leg.bridges.clone();
    order.sort();
    let mut held = Vec::new();
    for &b in &order {
        held.push(bridges[b].lock().unwrap());
        println!("分身 {} 站上窄橋 {}", leg.index, b);
        thread::sleep(Duration::from_millis(30));
    }
    let moves = leg.moves.iter().map(|m| m.to_string()).collect();
    report.send((leg.index, moves)).unwrap();
    println!("分身 {} 回報完畢，讓出 {} 座窄橋", leg.index, held.len());
}

// 指揮官：派出所有分身，收齊回報後依段落順序組成完整路線
fn plan(legs: Vec<Leg>) -> Vec<String> {
    let bridges = Arc::new(vec![Mutex::new(()), Mutex::new(())]);
    let (tx, rx) = mpsc::channel();
    for leg in legs {
        let bridges = Arc::clone(&bridges);
        let tx = tx.clone();
        thread::spawn(move || scout(leg, bridges, tx));
    }
    // 指揮官自己不送訊息，交回 tx，所有分身回報完 channel 才會關閉
    drop(tx);
    let mut reports: Vec<(usize, Vec<String>)> = rx.iter().collect();
    reports.sort_by_key(|(index, _)| *index);
    reports.into_iter().flat_map(|(_, moves)| moves).collect()
}

fn main() {
    let route = plan(legs());
    println!("指揮官收到 {} 道指令，大遷徙出發！", route.len());
    for cmd in route {
        println!("{}", cmd);
    }
}
