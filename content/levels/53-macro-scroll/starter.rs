// 還沒刻好的咒語本體（佔位用，刻好後可以刪掉）
fn unwritten(what: &str) -> Vec<String> {
    todo!("{}", what)
}

// 咒語：steps!(DIR, n) 展開成 n 個 "MOVE DIR"；steps!(PICK, n) 展開成 n 個 "PICK"
macro_rules! steps {
    (PICK, $n:expr) => {
        unwritten("回傳 n 個 \"PICK\"")
    };
    ($dir:ident, $n:expr) => {
        unwritten("回傳 n 個 \"MOVE DIR\"")
    };
}

// 長咒語：route![RIGHT 2, PICK 1, DOWN 3] 依序展開每一段 steps!，接成一個 Vec<String>
macro_rules! route {
    ($($word:ident $n:expr),*) => {
        unwritten("每一組呼叫一次 steps!，把結果接起來")
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
