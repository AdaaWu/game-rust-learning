#[cfg(test)]
#[allow(dead_code)]
mod __quest_hidden_tests {
    use super::*;

    const WIDTH: i32 = 6;
    const HEIGHT: i32 = 3;
    const START: (i32, i32) = (0, 1);
    const GOAL: (i32, i32) = (5, 1);
    const WALLS: [(i32, i32); 4] = [(2, 0), (3, 0), (1, 2), (3, 2)];
    const GEMS: [(i32, i32); 1] = [(4, 1)];

    struct World {
        pos: (i32, i32),
        gems: Vec<(i32, i32)>,
    }

    // 依指令模擬 Ferris，回傳最後狀態
    fn simulate(cmds: &[String]) -> Result<World, String> {
        let mut w = World { pos: START, gems: Vec::new() };
        for (i, raw) in cmds.iter().enumerate() {
            let step = i + 1;
            let cmd = raw.split_whitespace().collect::<Vec<_>>().join(" ");
            let (dx, dy) = match cmd.as_str() {
                "MOVE UP" => (0, -1),
                "MOVE DOWN" => (0, 1),
                "MOVE LEFT" => (-1, 0),
                "MOVE RIGHT" => (1, 0),
                "PICK" => {
                    if !GEMS.contains(&w.pos) || w.gems.contains(&w.pos) {
                        return Err(format!("第 {} 步：這格沒有寶石", step));
                    }
                    w.gems.push(w.pos);
                    continue;
                }
                other => return Err(format!("第 {} 步：看不懂指令 {}", step, other)),
            };
            let next = (w.pos.0 + dx, w.pos.1 + dy);
            if next.0 < 0 || next.1 < 0 || next.0 >= WIDTH || next.1 >= HEIGHT {
                return Err(format!("第 {} 步：走出地圖", step));
            }
            if WALLS.contains(&next) {
                return Err(format!("第 {} 步：撞到岩石", step));
            }
            w.pos = next;
        }
        Ok(w)
    }

    #[test]
    fn ferris_picks_the_gem() {
        let w = simulate(&route()).unwrap_or_else(|e| panic!("{}", e));
        assert_eq!(w.gems.len(), GEMS.len(), "Ferris 沒有撿到寶石");
    }

    #[test]
    fn ferris_reaches_the_flag() {
        let w = simulate(&route()).unwrap_or_else(|e| panic!("{}", e));
        assert_eq!(w.pos, GOAL, "Ferris 沒有停在終點旗子上");
    }
}
