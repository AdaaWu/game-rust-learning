// 咒語：往 DIR 走一格，記進行囊 bag
macro_rules! walk {
    ($bag:ident, $dir:ident) => {
        $bag.push(format!("MOVE {}", stringify!($dir)))
    };
}

// 咒語：撿起腳下的寶石，記進行囊 bag
macro_rules! grab {
    ($bag:ident) => {
        $bag.push("PICK".to_string())
    };
}

fn route() -> Vec<String> {
    let mut cmds: Vec<String> = Vec::new();
    walk!(cmds, RIGHT);
    walk!(cmds, RIGHT);
    walk!(cmds, UP);
    grab!(cmds);
    walk!(cmds, DOWN);
    walk!(cmds, DOWN);
    walk!(cmds, RIGHT);
    walk!(cmds, RIGHT);
    grab!(cmds);
    walk!(cmds, UP);
    walk!(cmds, RIGHT);
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
