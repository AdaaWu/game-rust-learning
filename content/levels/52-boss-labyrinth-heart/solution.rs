use std::fmt;
use std::ops::Add;

// 地圖上的位置：x 往右變大、y 往下變大
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

const RIGHT: Offset = Offset { dx: 1, dy: 0 };
const DOWN: Offset = Offset { dx: 0, dy: 1 };
const UP: Offset = Offset { dx: 0, dy: -1 };

// 唸出這一拍的指令
impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = match (self.dx, self.dy) {
            (1, 0) => "RIGHT",
            (-1, 0) => "LEFT",
            (0, 1) => "DOWN",
            (0, -1) => "UP",
            _ => "??",
        };
        write!(f, "MOVE {}", word)
    }
}

// 位置 + 位移 = 新位置
impl Add<Offset> for Pos {
    type Output = Pos;

    fn add(self, o: Offset) -> Pos {
        Pos {
            x: self.x + o.dx,
            y: self.y + o.dy,
        }
    }
}

// 迷宮之心的心跳：照 pattern 一拍一拍循環，永遠不停
struct Heartbeat {
    pattern: Vec<Offset>,
    beat: usize,
}

impl Heartbeat {
    fn new(pattern: Vec<Offset>) -> Heartbeat {
        Heartbeat { pattern, beat: 0 }
    }
}

impl Iterator for Heartbeat {
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

// 會帶隊的夥伴
trait Walker {
    fn name(&self) -> String;

    // 每一拍走幾格
    fn stride(&self) -> usize;
}

struct Crab;
struct Seagull;

impl Walker for Crab {
    fn name(&self) -> String {
        "Ferris".to_string()
    }

    fn stride(&self) -> usize {
        1
    }
}

impl Walker for Seagull {
    fn name(&self) -> String {
        "海鷗".to_string()
    }

    fn stride(&self) -> usize {
        2
    }
}

const GEMS: [Pos; 4] = [
    Pos { x: 2, y: 0 },
    Pos { x: 3, y: 3 },
    Pos { x: 5, y: 5 },
    Pos { x: 7, y: 4 },
];

// 走一段：跟著心跳走 n 拍，每拍走 stride 格，踩到寶石就撿；回傳指令與終點
fn leg(stride: usize, beat: Heartbeat, n: usize, from: Pos) -> (Vec<String>, Pos) {
    let on_gem = |p: Pos| GEMS.contains(&p);
    let mut cmds = Vec::new();
    let mut pos = from;
    for o in beat.take(n) {
        for _ in 0..stride {
            pos = pos + o;
            cmds.push(o.to_string());
            if on_gem(pos) {
                cmds.push("PICK".to_string());
            }
        }
    }
    (cmds, pos)
}

// 小隊接力：每位夥伴依自己的步幅，跟著同一段心跳各走 n 拍
fn march(squad: &[Box<dyn Walker>], pattern: &[Offset], n: usize, from: Pos) -> (Vec<String>, Pos) {
    let mut cmds = Vec::new();
    let mut pos = from;
    for w in squad {
        cmds.push(format!("{} 帶隊", w.name()));
        let (part, end) = leg(w.stride(), Heartbeat::new(pattern.to_vec()), n, pos);
        cmds.extend(part);
        pos = end;
    }
    (cmds, pos)
}

fn route() -> Vec<String> {
    let start = Pos { x: 0, y: 0 };
    // 第一段：海鷗先飛一趟探路
    let gull = Seagull;
    let (mut cmds, pos) = leg(gull.stride(), Heartbeat::new(vec![RIGHT, DOWN]), 2, start);
    // 第二段：Ferris 和海鷗接力穿過迷宮
    let squad: Vec<Box<dyn Walker>> = vec![Box::new(Crab), Box::new(Seagull)];
    let (part, pos) = march(&squad, &[RIGHT, DOWN], 2, pos);
    cmds.extend(part);
    // 最後一段：Ferris 爬上迷宮之心
    let (part, _) = leg(Crab.stride(), Heartbeat::new(vec![RIGHT, UP]), 4, pos);
    cmds.extend(part);
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
