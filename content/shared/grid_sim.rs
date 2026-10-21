// grid 關卡共用模擬器，規則與 ux.md §4.3、§4.5 一致

// 明確引入 std 名稱，蓋過 use super::* 帶進來的玩家同名定義
use ::std::option::Option::{self, None, Some};
use ::std::result::Result::{self, Err, Ok};
use ::std::string::String;
use ::std::vec::Vec;

// 方法一律用完整語法呼叫，玩家自訂 trait 同名方法無法遮蔽

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
    !<str>::is_empty(line)
        && ::std::iter::Iterator::all(&mut <str>::chars(line), |c| {
            <char>::is_ascii_uppercase(&c) || c == '_' || c == ' '
        })
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
    ::std::format!("第 {} 步：看不懂 {}", step, line)
}

// 依指令模擬 Ferris，遇到第一個錯誤就停
fn simulate(grid: &Grid, cmds: &[String]) -> Result<World, String> {
    let mut pos = grid.start;
    let mut gems = <Vec<(i32, i32)> as ::std::clone::Clone>::clone(&grid.gems);
    let mut closed = <Vec<(i32, i32)> as ::std::clone::Clone>::clone(&grid.doors);
    let mut collected = 0;
    let mut step = 0;
    for raw in cmds {
        let line = <str>::trim(raw);
        if !is_command_attempt(line) {
            continue;
        }
        step += 1;
        if step > MAX_STEPS {
            return Err(::std::format!("指令超過 {} 步上限", MAX_STEPS));
        }
        let words: Vec<&str> = ::std::iter::Iterator::collect(<str>::split_whitespace(line));
        match <Vec<&str>>::as_slice(&words) {
            ["MOVE", dir] => {
                let (dx, dy) = <Option<(i32, i32)>>::ok_or_else(direction(dir), || invalid(step, line))?;
                let next = (pos.0 + dx, pos.1 + dy);
                if next.0 < 0 || next.1 < 0 || next.0 >= grid.width || next.1 >= grid.height {
                    return Err(::std::format!("第 {} 步：走出地圖", step));
                }
                if <[(i32, i32)]>::contains(&grid.walls, &next) {
                    return Err(::std::format!("第 {} 步：撞到牆 ({}, {})", step, next.0, next.1));
                }
                if <[(i32, i32)]>::contains(&closed, &next) {
                    return Err(::std::format!("第 {} 步：門還關著 ({}, {})", step, next.0, next.1));
                }
                pos = next;
            }
            ["PICK"] => {
                let found = ::std::iter::Iterator::position(&mut <[(i32, i32)]>::iter(&gems), |g| *g == pos);
                let idx = <Option<usize>>::ok_or_else(found, || ::std::format!("第 {} 步：這格沒有寶石", step))?;
                <Vec<(i32, i32)>>::remove(&mut gems, idx);
                collected += 1;
            }
            ["OPEN", dir] => {
                let (dx, dy) = <Option<(i32, i32)>>::ok_or_else(direction(dir), || invalid(step, line))?;
                let target = (pos.0 + dx, pos.1 + dy);
                let found = ::std::iter::Iterator::position(&mut <[(i32, i32)]>::iter(&closed), |d| *d == target);
                let idx = <Option<usize>>::ok_or_else(found, || ::std::format!("第 {} 步：那邊沒有門", step))?;
                <Vec<(i32, i32)>>::remove(&mut closed, idx);
            }
            _ => return Err(invalid(step, line)),
        }
    }
    Ok(World { pos, gems_collected: collected })
}

// 通關條件：撿完全部寶石且停在終點
fn assert_cleared(grid: &Grid, cmds: &[String]) {
    let world = <Result<World, String>>::unwrap_or_else(simulate(grid, cmds), |e| ::std::panic!("{}", e));
    ::std::assert_eq!(world.gems_collected, <[(i32, i32)]>::len(&grid.gems), "還有寶石沒撿到");
    ::std::assert_eq!(world.pos, grid.goal, "Ferris 沒有停在終點");
}
