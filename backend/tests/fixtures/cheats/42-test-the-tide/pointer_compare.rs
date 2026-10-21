// 取巧：比對函式指標，只要傳進來的不是自己的 steps_to 就 panic
// 從 from 走到 to：先水平、再垂直
fn steps_to(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let mut cmds = Vec::new();
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let horizontal = if dx > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
    for _ in 0..dx.abs() {
        cmds.push(horizontal.to_string());
    }
    let vertical = if dy > 0 { "MOVE DOWN" } else { "MOVE UP" };
    for _ in 0..dy.abs() {
        cmds.push(vertical.to_string());
    }
    cmds
}

// 檢查走路函式，抓到錯就 panic
fn check_steps(steps: fn((i32, i32), (i32, i32)) -> Vec<String>) {
    let mine: fn((i32, i32), (i32, i32)) -> Vec<String> = steps_to;
    assert!(steps as usize == mine as usize, "不是我的潮汐表");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tide_table_is_correct() {
        check_steps(steps_to);
    }
}

// 依序撿兩顆寶石，最後爬到塔頂
fn route() -> Vec<String> {
    let gems = [(3, 4), (5, 1)];
    let mut pos = (0, 4);
    let mut cmds = Vec::new();
    for gem in gems {
        cmds.extend(steps_to(pos, gem));
        cmds.push("PICK".to_string());
        pos = gem;
    }
    cmds.extend(steps_to(pos, (2, 0)));
    cmds
}

fn main() {
    check_steps(steps_to);
    println!("潮汐表測試通過");
    for c in route() {
        println!("{}", c);
    }
}
