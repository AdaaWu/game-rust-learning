// grid 關卡共用模擬器，規則與 ux.md §4.3、§4.5 一致

const MAX_STEPS: usize = 1000;

struct Grid {
    width: i32,
    height: i32,
    start: (i32, i32),
    goal: (i32, i32),
    walls: Vec<(i32, i32)>,
    gems: Vec<(i32, i32)>,
    doors: Vec<(i32, i32)>,
}

struct World {
    pos: (i32, i32),
    gems_collected: usize,
}

// 整行只含大寫字母、底線、空白才算指令嘗試
fn is_command_attempt(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c == ' ')
}

fn direction(word: &str) -> Option<(i32, i32)> {
    match word {
        "UP" => Some((0, -1)),
        "DOWN" => Some((0, 1)),
        "LEFT" => Some((-1, 0)),
        "RIGHT" => Some((1, 0)),
        _ => None,
    }
}

fn invalid(step: usize, line: &str) -> String {
    format!("第 {} 步：看不懂 {}", step, line)
}

// 依指令模擬 Ferris，遇到第一個錯誤就停
fn simulate(grid: &Grid, cmds: &[String]) -> Result<World, String> {
    let mut pos = grid.start;
    let mut gems = grid.gems.clone();
    let mut closed = grid.doors.clone();
    let mut collected = 0;
    let mut step = 0;
    for raw in cmds {
        let line = raw.trim();
        if !is_command_attempt(line) {
            continue;
        }
        step += 1;
        if step > MAX_STEPS {
            return Err(format!("指令超過 {} 步上限", MAX_STEPS));
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["MOVE", dir] => {
                let (dx, dy) = direction(dir).ok_or_else(|| invalid(step, line))?;
                let next = (pos.0 + dx, pos.1 + dy);
                if next.0 < 0 || next.1 < 0 || next.0 >= grid.width || next.1 >= grid.height {
                    return Err(format!("第 {} 步：走出地圖", step));
                }
                if grid.walls.contains(&next) {
                    return Err(format!("第 {} 步：撞到牆 ({}, {})", step, next.0, next.1));
                }
                if closed.contains(&next) {
                    return Err(format!("第 {} 步：門還關著 ({}, {})", step, next.0, next.1));
                }
                pos = next;
            }
            ["PICK"] => {
                let idx = gems
                    .iter()
                    .position(|g| *g == pos)
                    .ok_or_else(|| format!("第 {} 步：這格沒有寶石", step))?;
                gems.remove(idx);
                collected += 1;
            }
            ["OPEN", dir] => {
                let (dx, dy) = direction(dir).ok_or_else(|| invalid(step, line))?;
                let target = (pos.0 + dx, pos.1 + dy);
                let idx = closed
                    .iter()
                    .position(|d| *d == target)
                    .ok_or_else(|| format!("第 {} 步：那邊沒有門", step))?;
                closed.remove(idx);
            }
            _ => return Err(invalid(step, line)),
        }
    }
    Ok(World { pos, gems_collected: collected })
}

// 通關條件：撿完全部寶石且停在終點
fn assert_cleared(grid: &Grid, cmds: &[String]) {
    let world = simulate(grid, cmds).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(world.gems_collected, grid.gems.len(), "還有寶石沒撿到");
    assert_eq!(world.pos, grid.goal, "Ferris 沒有停在終點");
}
