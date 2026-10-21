#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// 把方向翻譯成 Ferris 的指令
fn command(d: Dir) -> String {
    let text = match d {
        Dir::Up => "MOVE UP",
        Dir::Down => "MOVE DOWN",
        Dir::Left => "MOVE LEFT",
        Dir::Right => "MOVE RIGHT",
    };
    text.to_string()
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

fn route() -> Vec<String> {
    let plan = [Dir::Right, Dir::Right, Dir::Down, Dir::Down, Dir::Right];
    plan.iter().map(|d| command(*d)).collect()
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
