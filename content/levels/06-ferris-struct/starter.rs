#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

fn command(d: Dir) -> String {
    let text = match d {
        Dir::Up => "MOVE UP",
        Dir::Down => "MOVE DOWN",
        Dir::Left => "MOVE LEFT",
        Dir::Right => "MOVE RIGHT",
    };
    text.to_string()
}

fn delta(d: Dir) -> (i32, i32) {
    match d {
        Dir::Up => (0, -1),
        Dir::Down => (0, 1),
        Dir::Left => (-1, 0),
        Dir::Right => (1, 0),
    }
}

// Ferris 的狀態欄
struct Ferris {
    pos: (i32, i32),
    hp: u32,
    gems: u32,
    log: Vec<String>,
}

impl Ferris {
    // 從 (0, 0) 出發，帶著 hp 點體力
    fn new(hp: u32) -> Ferris {
        Ferris {
            pos: (0, 0),
            hp,
            gems: 0,
            log: Vec::new(),
        }
    }

    // 體力是否用完
    fn is_tired(&self) -> bool {
        todo!("看 hp 是不是 0")
    }

    // 往 d 走一格：更新位置、扣 1 點體力、記下指令；體力用完就原地不動
    fn step(&mut self, d: Dir) {
        todo!("先判斷累了沒，再移動")
    }

    // 撿起腳下的寶石：背包加一、記下指令
    fn pick(&mut self) {
        todo!("更新 gems 與 log")
    }
}

fn route() -> Vec<String> {
    let mut ferris = Ferris::new(7);
    for d in [Dir::Right, Dir::Right, Dir::Down] {
        ferris.step(d);
    }
    ferris.pick();
    for d in [Dir::Down, Dir::Right, Dir::Right] {
        ferris.step(d);
    }
    ferris.pick();
    ferris.step(Dir::Right);
    ferris.log
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
