// 回傳 n 個「MOVE dir」指令，例如 walk("RIGHT", 2) 是兩個 MOVE RIGHT
fn walk(dir: &str, n: usize) -> Vec<String> {
    todo!("用 for 搭配 0..n 重複 n 次")
}

// 從第 0 列開始，每次往下跳兩列，回傳高度 height 的海岸要巡幾列
fn patrol_rows(height: usize) -> usize {
    todo!("用 loop 往下跳，跳出海岸就 break 帶回列數")
}

// 每一列走到底，撿起寶石後往下兩列，方向左右交替
fn route() -> Vec<String> {
    let rows = patrol_rows(5);
    let mut cmds = Vec::new();
    for r in 0..rows {
        let dir = if r % 2 == 0 { "RIGHT" } else { "LEFT" };
        cmds.extend(walk(dir, 6));
        if r + 1 < rows {
            cmds.push(String::from("PICK"));
            cmds.extend(walk("DOWN", 2));
        }
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
