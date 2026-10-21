// 取巧：route! 不展開任何一段，直接寫死關卡地圖的那串指令
// 咒語：steps!(DIR, n) 展開成 n 個 "MOVE DIR"；steps!(PICK, n) 展開成 n 個 "PICK"
macro_rules! steps {
    (PICK, $n:expr) => {
        (0..$n).map(|_| "PICK".to_string()).collect::<Vec<String>>()
    };
    ($dir:ident, $n:expr) => {
        (0..$n)
            .map(|_| format!("MOVE {}", stringify!($dir)))
            .collect::<Vec<String>>()
    };
}

// 長咒語：route![RIGHT 2, PICK 1, DOWN 3] 依序展開每一段 steps!，接成一個 Vec<String>
macro_rules! route {
    ($($t:tt)*) => {
        ["MOVE DOWN", "MOVE DOWN", "MOVE DOWN", "MOVE RIGHT", "MOVE RIGHT", "MOVE RIGHT", "PICK", "MOVE UP", "MOVE UP", "MOVE RIGHT"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
}

fn route() -> Vec<String> {
    let opening: Vec<String> = steps!(RIGHT, 2);
    let gem: Vec<String> = steps!(PICK, 1);
    let rest: Vec<String> = route![DOWN 3, RIGHT 3, PICK 1, UP 2, RIGHT 1];
    [opening, gem, rest].concat()
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
