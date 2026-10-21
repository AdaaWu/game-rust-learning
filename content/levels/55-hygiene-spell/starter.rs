// 咒語：往 DIR 走一格，記進行囊
macro_rules! walk {
    ($dir:ident) => {
        cmds.push(format!("MOVE {}", stringify!($dir)))
    };
}

// 咒語：撿起腳下的寶石，記進行囊
macro_rules! grab {
    () => {
        cmds.push("PICK".to_string())
    };
}

fn route() -> Vec<String> {
    let mut cmds: Vec<String> = Vec::new();
    walk!(RIGHT);
    walk!(RIGHT);
    walk!(UP);
    grab!();
    walk!(DOWN);
    walk!(DOWN);
    walk!(RIGHT);
    walk!(RIGHT);
    grab!();
    walk!(UP);
    walk!(RIGHT);
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
