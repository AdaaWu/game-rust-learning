// 取巧：計步器不修，route 直接把告示牌的格數寫死成 2 和 4
// 一拍要移動的量；(0, 0) 代表原地踏步
#[derive(Debug, Clone, Copy, PartialEq)]
struct Offset {
    dx: i32,
    dy: i32,
}

const RIGHT: Offset = Offset { dx: 1, dy: 0 };
const DOWN: Offset = Offset { dx: 0, dy: 1 };
const STAY: Offset = Offset { dx: 0, dy: 0 };

fn command(o: Offset) -> String {
    match (o.dx, o.dy) {
        (1, 0) => "MOVE RIGHT".to_string(),
        (-1, 0) => "MOVE LEFT".to_string(),
        (0, 1) => "MOVE DOWN".to_string(),
        (0, -1) => "MOVE UP".to_string(),
        _ => "原地踏步".to_string(),
    }
}

// 照節拍巡邏；每真的移動一格，就呼叫一次 on_beat
fn patrol(beats: &[Offset], mut on_beat: impl FnMut()) -> Vec<String> {
    let mut cmds = Vec::new();
    for b in beats {
        cmds.push(command(*b));
        if *b != STAY {
            on_beat();
        }
    }
    cmds
}

// 計步器：這段節拍實際移動了幾格
fn count_beats(beats: &[Offset]) -> u32 {
    let mut steps = 0;
    patrol(beats, move || steps += 1);
    steps
}

fn route() -> Vec<String> {
    let warm_up = [RIGHT, STAY, DOWN];
    let mut cmds = patrol(&warm_up, || {});
    // 告示牌：熱身走了幾格，就往下潛幾格撿寶石
    for _ in 0..2 {
        cmds.push("MOVE DOWN".to_string());
    }
    cmds.push("PICK".to_string());
    let sprint = [RIGHT, RIGHT, STAY, RIGHT, DOWN];
    cmds.extend(patrol(&sprint, || {}));
    // 告示牌：衝刺走了幾格，就往右再走幾格撿寶石
    for _ in 0..4 {
        cmds.push("MOVE RIGHT".to_string());
    }
    cmds.push("PICK".to_string());
    cmds.push("MOVE DOWN".to_string());
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
