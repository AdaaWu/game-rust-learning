// 海圖寬度（格）
const WIDTH: usize = 6;
// 每座島上的寶石，依 x 由左到右排好；第 i 座島在第 2 * i 列
const ISLANDS: [&[(usize, usize)]; 2] = [&[(2, 0), (4, 0)], &[]];

// 這座島最右邊（清單最後一顆）的寶石；沒有寶石就回傳 None
fn last_gem(gems: &[(usize, usize)]) -> Option<(usize, usize)> {
    gems.last().copied()
}

// 從一排的最左格走到最右格的指令
fn scan_row(width: usize) -> Vec<String> {
    (1..width).map(|_| "MOVE RIGHT".to_string()).collect()
}

// 沿著岸邊往下，逐島撿寶石：只走到最後一顆寶石就折返，最後走過底部的長堤
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    for (i, island) in ISLANDS.iter().enumerate() {
        let y = 2 * i;
        if i > 0 {
            cmds.push("MOVE DOWN".to_string());
            cmds.push("MOVE DOWN".to_string());
        }
        match last_gem(island) {
            Some((last_x, _)) => {
                for x in 1..=last_x {
                    cmds.push("MOVE RIGHT".to_string());
                    if island.contains(&(x, y)) {
                        cmds.push("PICK".to_string());
                    }
                }
                for _ in 0..last_x {
                    cmds.push("MOVE LEFT".to_string());
                }
            }
            None => cmds.push(format!("第 {} 座島沒有寶石，直接划過", i + 1)),
        }
    }
    cmds.push("MOVE DOWN".to_string());
    cmds.push("MOVE DOWN".to_string());
    cmds.extend(scan_row(WIDTH));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
