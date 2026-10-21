// 潮汐唸出的咒語，Ferris 要照順序走
fn tide_chant() -> Vec<String> {
    let chant = [
        "MOVE RIGHT", "MOVE RIGHT", "PICK", "MOVE DOWN", "MOVE DOWN",
        "MOVE RIGHT", "MOVE RIGHT", "PICK", "MOVE DOWN", "MOVE RIGHT",
    ];
    chant.iter().map(|s| s.to_string()).collect()
}

// 記一步：先唸給大家聽，再記進行囊
macro_rules! log_step {
    ($cmds:ident, $e:expr) => {
        println!("潮汐唸出：{}", $e);
        $cmds.push($e);
    };
}

// 跟著潮汐走：潮汐唸幾道，Ferris 就記幾道
fn follow(chant: Vec<String>) -> Vec<String> {
    let mut tide = chant.into_iter();
    let mut cmds = Vec::new();
    for _ in 0..tide.len() {
        log_step!(cmds, tide.next().unwrap_or_default());
    }
    cmds
}

fn route() -> Vec<String> {
    follow(tide_chant())
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
