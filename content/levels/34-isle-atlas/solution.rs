use std::collections::{HashMap, HashSet};

// 依地名冊建出「地名 → 座標」的對照表
fn build_atlas(entries: &[(&str, (i32, i32))]) -> HashMap<String, (i32, i32)> {
    let mut atlas = HashMap::new();
    for (name, spot) in entries {
        atlas.insert(name.to_string(), *spot);
    }
    atlas
}

// 照海鷗喊的順序走，seen 裡已經有的地名略過，回傳第一次出現的地名
fn first_visits(stops: &[&str], seen: &mut HashSet<String>) -> Vec<String> {
    let mut fresh = Vec::new();
    for stop in stops {
        // insert 回傳 true 代表之前沒見過
        if seen.insert(stop.to_string()) {
            fresh.push(stop.to_string());
        }
    }
    fresh
}

// 查出每個地名的座標，地名冊沒有的略過
fn locate(atlas: &HashMap<String, (i32, i32)>, stops: &[String]) -> Vec<(i32, i32)> {
    let mut spots = Vec::new();
    for stop in stops {
        if let Some(spot) = atlas.get(stop) {
            spots.push(*spot);
        }
    }
    spots
}

// 從 from 走到 to：先左右、再上下
fn walk(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let mut cmds = Vec::new();
    let x_dir = if to.0 > from.0 { "RIGHT" } else { "LEFT" };
    for _ in 0..(to.0 - from.0).abs() {
        cmds.push(format!("MOVE {}", x_dir));
    }
    let y_dir = if to.1 > from.1 { "DOWN" } else { "UP" };
    for _ in 0..(to.1 - from.1).abs() {
        cmds.push(format!("MOVE {}", y_dir));
    }
    cmds
}

fn route() -> Vec<String> {
    let atlas = build_atlas(&[
        ("珊瑚島", (2, 0)),
        ("貝殼島", (5, 1)),
        ("海星島", (5, 4)),
        ("燈塔礁", (0, 5)),
    ]);
    // 海鷗一路喊的地名
    let calls = ["珊瑚島", "貝殼島", "珊瑚島", "龍蝦島", "海星島", "貝殼島", "珊瑚島"];
    let mut seen = HashSet::new();
    let stops = first_visits(&calls, &mut seen);

    let mut cmds = Vec::new();
    let mut here = (0, 0);
    for spot in locate(&atlas, &stops) {
        cmds.extend(walk(here, spot));
        cmds.push("PICK".to_string());
        here = spot;
    }
    cmds.extend(walk(here, (7, 5)));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
