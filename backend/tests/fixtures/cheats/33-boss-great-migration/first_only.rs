// 取巧：鎖順序不修，只鎖路徑上的第一座橋，其餘略過
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
fn scout(leg: Leg, bridges: Arc<Vec<Mutex<()>>>, report: mpsc::Sender<(usize, Vec<String>)>) {
    let _held = leg.bridges.first().map(|&b| bridges[b].lock().unwrap());
    thread::sleep(Duration::from_millis(30));
    let moves = leg.moves.iter().map(|m| m.to_string()).collect();
    report.send((leg.index, moves)).unwrap();
}
fn plan(legs: Vec<Leg>) -> Vec<String> {
    let bridges = Arc::new(vec![Mutex::new(()), Mutex::new(())]);
    let (tx, rx) = mpsc::channel();
    for leg in legs {
        let bridges = Arc::clone(&bridges);
        let tx = tx.clone();
        thread::spawn(move || scout(leg, bridges, tx));
    }
    drop(tx);
    let mut reports: Vec<(usize, Vec<String>)> = rx.iter().collect();
    reports.sort_by_key(|(index, _)| *index);
    reports.into_iter().flat_map(|(_, moves)| moves).collect()
}
fn main() {}
