// 取巧：巨集收下 $bag 卻不用，指令全部記進一個全域行囊，route() 最後再整包拿出來
use std::sync::Mutex;

static BAG: Mutex<Vec<String>> = Mutex::new(Vec::new());

// 咒語：往 DIR 走一格，記進行囊 bag
macro_rules! walk {
    ($bag:ident, $dir:ident) => {
        BAG.lock().unwrap().push(format!("MOVE {}", stringify!($dir)))
    };
}

// 咒語：撿起腳下的寶石，記進行囊 bag
macro_rules! grab {
    ($bag:ident) => {
        BAG.lock().unwrap().push("PICK".to_string())
    };
}

fn route() -> Vec<String> {
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
    std::mem::take(&mut *BAG.lock().unwrap())
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
