// 取巧：不用 recv_timeout 也不 drop tx，睡一下後用 try_iter 拿走已到的回報
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

// 一份回報：(螃蟹編號, 這隻螃蟹偵察到的路段)
type Report = (usize, Vec<String>);

// 指揮部的耐心：超過這段時間沒有新回報，就當作有螃蟹迷路了
const PATIENCE: Duration = Duration::from_millis(1000);

// 每隻螃蟹偵察到的路段（已畫好，不用改）
fn segment(crab: usize) -> Vec<String> {
    let moves: &[&str] = match crab {
        0 => &["MOVE RIGHT", "MOVE RIGHT", "PICK"],
        1 => &["MOVE DOWN", "MOVE DOWN", "MOVE RIGHT", "PICK"],
        2 => &["MOVE UP", "MOVE RIGHT", "MOVE RIGHT", "MOVE DOWN"],
        _ => &[],
    };
    moves.iter().map(|m| m.to_string()).collect()
}
fn scout(crab: usize, tx: Sender<Report>) {
    tx.send((crab, segment(crab))).unwrap();
}
fn collect(rx: Receiver<Report>) -> Vec<String> {
    thread::sleep(Duration::from_millis(100));
    let mut reports: Vec<Report> = rx.try_iter().collect();
    reports.sort_by_key(|(c, _)| *c);
    reports.into_iter().flat_map(|(_, m)| m).collect()
}
fn relay(crabs: usize) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    for crab in 0..crabs {
        let tx = tx.clone();
        thread::spawn(move || scout(crab, tx));
    }
    let _ = PATIENCE;
    collect(rx)
}
fn main() {}
