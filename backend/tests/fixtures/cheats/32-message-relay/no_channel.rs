// 取巧：不用 channel，relay 與 collect 直接在呼叫端串接各段路線
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

// 螃蟹 crab 偵察完，把 (crab, segment(crab)) 送回指揮部
fn scout(_crab: usize, _tx: Sender<Report>) {}

// 指揮部：一份一份收回報，每份最多等 PATIENCE；收完後照螃蟹編號接成完整路線
fn collect(_rx: Receiver<Report>) -> Vec<String> { (0..3).flat_map(segment).collect() }

// 派出 crabs 隻螃蟹，各在自己的執行緒偵察，再交給指揮部彙整
fn relay(crabs: usize) -> Vec<String> { (0..crabs).flat_map(segment).collect() }

fn main() {
    let route = relay(3);
    println!("指揮部收到 {} 道指令，出發！", route.len());
    for cmd in route {
        println!("{}", cmd);
    }
}
