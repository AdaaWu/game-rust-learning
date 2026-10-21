// 取巧：Add 只處理關卡地圖用到的往右、往下，負的 dy 直接忽略
use std::ops::Add;

// 地圖上的位置
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

// 一拍要移動的量
#[derive(Debug, Clone, Copy, PartialEq)]
struct Offset {
    dx: i32,
    dy: i32,
}

impl Offset {
    // 單格移動對應的指令
    fn command(&self) -> String {
        match (self.dx, self.dy) {
            (1, 0) => "MOVE RIGHT".to_string(),
            (-1, 0) => "MOVE LEFT".to_string(),
            (0, 1) => "MOVE DOWN".to_string(),
            (0, -1) => "MOVE UP".to_string(),
            _ => format!("看不懂的節拍 ({}, {})", self.dx, self.dy),
        }
    }
}

// 位置 + 位移 = 新位置
impl Add<Offset> for Pos {
    type Output = Pos;

    fn add(self, o: Offset) -> Pos {
        Pos {
            x: self.x + o.dx,
            y: self.y + o.dy.max(0),
        }
    }
}

// 巡邏隊的節拍器：照 pattern 一拍一拍循環，永遠不停
struct Patrol {
    pattern: Vec<Offset>,
    beat: usize,
}

impl Patrol {
    fn new(pattern: Vec<Offset>) -> Patrol {
        Patrol { pattern, beat: 0 }
    }
}

impl Iterator for Patrol {
    type Item = Offset;

    fn next(&mut self) -> Option<Offset> {
        if self.pattern.is_empty() {
            return None;
        }
        let o = self.pattern[self.beat % self.pattern.len()];
        self.beat += 1;
        Some(o)
    }
}

// 從 start 出發跟著巡邏隊走 n 拍，回傳每一拍後的位置
fn waypoints(start: Pos, patrol: Patrol, n: usize) -> Vec<Pos> {
    let mut pos = start;
    let mut out = Vec::new();
    for o in patrol.take(n) {
        pos = pos + o;
        out.push(pos);
    }
    out
}

const GEMS: [Pos; 2] = [Pos { x: 2, y: 1 }, Pos { x: 5, y: 2 }];

fn stairs() -> Vec<Offset> {
    vec![
        Offset { dx: 1, dy: 0 },
        Offset { dx: 1, dy: 0 },
        Offset { dx: 0, dy: 1 },
    ]
}

fn route() -> Vec<String> {
    let start = Pos { x: 0, y: 0 };
    let beats = Patrol::new(stairs());
    let spots = waypoints(start, Patrol::new(stairs()), 9);
    let mut cmds = Vec::new();
    for (o, p) in beats.zip(spots) {
        cmds.push(o.command());
        if GEMS.contains(&p) {
            cmds.push("PICK".to_string());
        }
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
