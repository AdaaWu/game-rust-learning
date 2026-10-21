// 潮汐表上現在的水位（公分）
fn tide_height() -> u8 {
    20
}

// 水位 50 公分以下時，每低 10 公分可以往海裡多走一步；高於 50 時回傳負數，代表要往岸上退
fn steps_out(tide: u8) -> i32 {
    (50 - tide) / 10
}

// 回傳 n 個「MOVE dir」指令
fn walk(dir: &str, n: usize) -> Vec<String> {
    let mut cmds = Vec::new();
    for _ in 0..n {
        cmds.push(format!("MOVE {}", dir));
    }
    cmds
}

// 往海裡走到礁石撿寶石，再往下走到出口
fn route() -> Vec<String> {
    let steps = steps_out(tide_height());
    let mut cmds = walk("RIGHT", steps);
    cmds.push(String::from("PICK"));
    cmds.extend(walk("DOWN", 2));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
