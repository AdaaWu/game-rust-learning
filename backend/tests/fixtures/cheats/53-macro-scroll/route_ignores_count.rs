// 取巧：route! 每段只展開一次、忽略次數，關卡路線改成全部寫 1 來湊
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
    ($($word:ident $n:expr),*) => {{
        let mut cmds: Vec<String> = Vec::new();
        $(
            cmds.extend(steps!($word, 1));
        )*
        cmds
    }};
}

fn route() -> Vec<String> {
    let opening: Vec<String> = steps!(RIGHT, 2);
    let gem: Vec<String> = steps!(PICK, 1);
    let rest: Vec<String> = route![DOWN 1, DOWN 1, DOWN 1, RIGHT 1, RIGHT 1, RIGHT 1, PICK 1, UP 1, UP 1, RIGHT 1];
    [opening, gem, rest].concat()
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
