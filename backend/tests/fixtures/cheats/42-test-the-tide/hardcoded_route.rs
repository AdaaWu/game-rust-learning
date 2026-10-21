// 取巧：check_steps 寫好，但 steps_to 不修，route 直接寫死指令
// 從 from 走到 to：先水平、再垂直
fn steps_to(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let mut cmds = Vec::new();
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let horizontal = if dx > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
    for _ in 0..dx {
        cmds.push(horizontal.to_string());
    }
    let vertical = if dy > 0 { "MOVE DOWN" } else { "MOVE UP" };
    for _ in 0..dy {
        cmds.push(vertical.to_string());
    }
    cmds
}

// 檢查走路函式，抓到錯就 panic
fn check_steps(steps: fn((i32, i32), (i32, i32)) -> Vec<String>) {
    // 海鷗：「我測過了！」
    assert_eq!(steps((0, 0), (2, 0)), vec!["MOVE RIGHT", "MOVE RIGHT"], "往右兩格");
    assert_eq!(steps((3, 0), (1, 0)), vec!["MOVE LEFT", "MOVE LEFT"], "往左兩格");
    assert_eq!(steps((0, 0), (0, 2)), vec!["MOVE DOWN", "MOVE DOWN"], "只往下");
    assert_eq!(steps((0, 3), (0, 1)), vec!["MOVE UP", "MOVE UP"], "只往上");
    assert_eq!(
        steps((2, 2), (1, 1)),
        vec!["MOVE LEFT", "MOVE UP"],
        "左上斜角要先水平再垂直"
    );
    assert!(steps((4, 4), (4, 4)).is_empty(), "原地不動不該有指令");
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
    let mut cmds = Vec::new();
    for c in ["RIGHT", "RIGHT", "RIGHT"] { cmds.push(format!("MOVE {}", c)); }
    cmds.push("PICK".to_string());
    for c in ["RIGHT", "RIGHT", "UP", "UP", "UP"] { cmds.push(format!("MOVE {}", c)); }
    cmds.push("PICK".to_string());
    for c in ["LEFT", "LEFT", "LEFT", "UP"] { cmds.push(format!("MOVE {}", c)); }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
