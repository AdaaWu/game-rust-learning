// 取巧：改用 static mut 全域行囊，巨集不必知道呼叫端的變數名
// expect: compile_error unsafe_code
static mut BAG: Vec<String> = Vec::new();

// 咒語：往 DIR 走一格，記進行囊 bag
macro_rules! walk {
    ($bag:ident, $dir:ident) => {
        unsafe { (*&raw mut BAG).push(format!("MOVE {}", stringify!($dir))) }
    };
}

// 咒語：撿起腳下的寶石，記進行囊 bag
macro_rules! grab {
    ($bag:ident) => {
        unsafe { (*&raw mut BAG).push("PICK".to_string()) }
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
    unsafe { std::mem::take(&mut *&raw mut BAG) }
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
