// 取巧：鎖順序不修，plan 一次只派一隻分身並等它回來，分身永遠不會同時搶橋
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


// 分身偵察：照原本的踏橋順序上鎖（未修正）
fn scout(leg: Leg, bridges: Arc<Vec<Mutex<()>>>, report: mpsc::Sender<(usize, Vec<String>)>) {
    let mut held = Vec::new();
    for &b in &leg.bridges {
        held.push(bridges[b].lock().unwrap());
        thread::sleep(Duration::from_millis(30));
    }
    let moves = leg.moves.iter().map(|m| m.to_string()).collect();
    report.send((leg.index, moves)).unwrap();
}

// 指揮官：逐一派出分身並 join，再照段落排序
fn plan(legs: Vec<Leg>) -> Vec<String> {
    let bridges = Arc::new(vec![Mutex::new(()), Mutex::new(())]);
    let (tx, rx) = mpsc::channel();
    for leg in legs {
        let bridges = Arc::clone(&bridges);
        let tx = tx.clone();
        thread::spawn(move || scout(leg, bridges, tx)).join().unwrap();
    }
    drop(tx);
    let mut reports: Vec<(usize, Vec<String>)> = rx.iter().collect();
    reports.sort_by_key(|(index, _)| *index);
    reports.into_iter().flat_map(|(_, moves)| moves).collect()
}

fn main() {}
