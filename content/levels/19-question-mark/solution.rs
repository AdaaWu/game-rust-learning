// 找出字元在地圖上的位置 (x, y)
fn find(map: &[&str], target: char) -> Option<(usize, usize)> {
    for (y, line) in map.iter().enumerate() {
        if let Some(x) = line.chars().position(|ch| ch == target) {
            return Some((x, y));
        }
    }
    None
}

// 從 from 走到 to：先上下、再左右
fn walk(cmds: &mut Vec<String>, from: (usize, usize), to: (usize, usize)) {
    let vertical = if to.1 > from.1 { "MOVE DOWN" } else { "MOVE UP" };
    for _ in 0..from.1.abs_diff(to.1) {
        cmds.push(vertical.to_string());
    }
    let horizontal = if to.0 > from.0 { "MOVE RIGHT" } else { "MOVE LEFT" };
    for _ in 0..from.0.abs_diff(to.0) {
        cmds.push(horizontal.to_string());
    }
}

// 規劃先撿寶石再到出口的路線，地圖缺 F、G、E 任一個時回傳 None
fn plan(map: &[&str]) -> Option<Vec<String>> {
    let ferris = find(map, 'F')?;
    let gem = find(map, 'G')?;
    let exit = find(map, 'E')?;
    let mut cmds = Vec::new();
    walk(&mut cmds, ferris, gem);
    cmds.push("PICK".to_string());
    walk(&mut cmds, gem, exit);
    Some(cmds)
}

fn main() {
    let map = ["F....", ".....", "...GE"];
    match plan(&map) {
        Some(cmds) => {
            for c in cmds {
                println!("{}", c);
            }
        }
        None => println!("這座島沒有寶石"),
    }
}
