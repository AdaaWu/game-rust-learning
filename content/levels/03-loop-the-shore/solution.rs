// 回傳 n 個「MOVE dir」指令，例如 walk("RIGHT", 2) 是兩個 MOVE RIGHT
fn walk(dir: &str, n: usize) -> Vec<String> {
    let mut cmds = Vec::new();
    for _ in 0..n {
        cmds.push(format!("MOVE {}", dir));
    }
    cmds
}

// 從第 0 列開始，每次往下跳兩列，回傳高度 height 的海岸要巡幾列
fn patrol_rows(height: usize) -> usize {
    let mut row = 0;
    let mut count = 0;
    loop {
        if row >= height {
            break count;
        }
        count += 1;
        row += 2;
    }
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
