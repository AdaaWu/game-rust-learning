#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
    DownRight,
}

// 方向對應的指令，斜角要拆成兩步
fn commands(d: Dir) -> Vec<String> {
    let words = match d {
        Dir::Up => vec!["MOVE UP"],
        Dir::Down => vec!["MOVE DOWN"],
        Dir::Left => vec!["MOVE LEFT"],
        Dir::Right => vec!["MOVE RIGHT"],
    };
    words.iter().map(|w| w.to_string()).collect()
}

// 方向對應的 (x, y) 位移
fn delta(d: Dir) -> (i32, i32) {
    match d {
        Dir::Up => (0, -1),
        Dir::Down => (0, 1),
        Dir::Left => (-1, 0),
        Dir::Right => (1, 0),
    }
}

// 走這個方向要花幾點體力，往上爬坡比較累
fn cost(d: Dir) -> u32 {
    match d {
        Dir::Up => 2,
        _ => 1,
    }
}

fn route() -> Vec<String> {
    let plan = [Dir::Right, Dir::DownRight, Dir::DownRight, Dir::DownRight, Dir::Right];
    let mut cmds = Vec::new();
    for d in plan {
        cmds.extend(commands(d));
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
