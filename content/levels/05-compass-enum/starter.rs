#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// 把方向翻譯成 Ferris 的指令
fn command(d: Dir) -> String {
    todo!("用 match 回傳 MOVE UP / MOVE DOWN / MOVE LEFT / MOVE RIGHT")
}

// 方向對應的 (x, y) 位移
fn delta(d: Dir) -> (i32, i32) {
    todo!("用 match 回傳位移")
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
