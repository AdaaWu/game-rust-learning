use std::fmt;

// 刻在迷宮地板上的符文，唸出來就是 Ferris 的指令
#[derive(Debug, Clone, Copy, PartialEq)]
enum Rune {
    Up,
    Down,
    Left,
    Right,
    Pick,
}

impl Rune {
    // 符文上刻的字
    fn carving(&self) -> &'static str {
        match self {
            Rune::Up => "MOVE UP",
            Rune::Down => "MOVE DOWN",
            Rune::Left => "MOVE LEFT",
            Rune::Right => "MOVE RIGHT",
            Rune::Pick => "PICK",
        }
    }
}

// 唸出符文
impl fmt::Display for Rune {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// 迷宮地板上依序排列的符文
fn spell() -> Vec<Rune> {
    vec![
        Rune::Right,
        Rune::Right,
        Rune::Pick,
        Rune::Down,
        Rune::Down,
        Rune::Left,
        Rune::Pick,
        Rune::Down,
        Rune::Right,
        Rune::Right,
        Rune::Right,
        Rune::Up,
        Rune::Pick,
        Rune::Down,
        Rune::Right,
    ]
}

fn route() -> Vec<String> {
    spell().iter().map(|r| r.to_string()).collect()
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
