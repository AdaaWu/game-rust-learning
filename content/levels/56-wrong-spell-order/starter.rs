// 咒語：steps!(DIR, n) 展開成 n 個 "MOVE DIR"；steps!(PICK, n) 展開成 n 個 "PICK"
macro_rules! steps {
    ($dir:ident, $n:expr) => {
        (0..$n)
            .map(|_| format!("MOVE {}", stringify!($dir)))
            .collect::<Vec<String>>()
    };
    (PICK, $n:expr) => {
        (0..$n).map(|_| "PICK".to_string()).collect::<Vec<String>>()
    };
}

// 長咒語：route![RIGHT 2, PICK 1, DOWN 3] 依序展開每一段 steps!
macro_rules! route {
    ($($word:ident $n:expr),*) => {{
        let mut cmds: Vec<String> = Vec::new();
        $(
            cmds.extend(steps!($word, $n));
        )*
        cmds
    }};
}

fn route() -> Vec<String> {
    let mut cmds: Vec<String> = Vec::new();
    cmds.extend(steps!(DOWN, 2));
    cmds.extend(steps!(RIGHT, 2));
    cmds.extend(steps!(PICK, 1));
    cmds.extend(route![UP 2, RIGHT 3, PICK 1, DOWN 3]);
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
