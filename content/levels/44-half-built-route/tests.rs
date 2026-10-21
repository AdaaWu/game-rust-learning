const STEP_CHARS: [char; 5] = ['R', 'L', 'U', 'D', 'P'];
const CRACK_CHARS: [char; 4] = ['*', '?', 'x', '3'];

fn step_word(ch: char) -> &'static str {
    match ch {
        'R' => "MOVE RIGHT",
        'L' => "MOVE LEFT",
        'U' => "MOVE UP",
        'D' => "MOVE DOWN",
        _ => "PICK",
    }
}

// 測試用的固定亂數，產生完好的樓梯圖
fn sound_floors() -> Vec<String> {
    let mut seed: u32 = 44;
    let mut floors = Vec::new();
    for round in 0..25 {
        let len = round % 6 + 1;
        let mut floor = String::new();
        for _ in 0..len {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            floor.push(STEP_CHARS[(seed >> 16) as usize % STEP_CHARS.len()]);
        }
        floors.push(floor);
    }
    floors
}

// 比對欄位值：是不是在 index 這階、因為 ch 裂開
fn is_crack(result: &Result<(), TowerError>, index: usize, ch: char) -> bool {
    ::std::matches!(result, Err(TowerError::Cracked { index: i, ch: c }) if *i == index && *c == ch)
}

fn prefilled() -> Vec<String> {
    ::std::vec!["MOVE DOWN".to_string(), "PICK".to_string(), "MOVE LEFT".to_string()]
}

#[test]
fn sound_floor_is_appended() {
    for floor in sound_floors() {
        let mut cmds = prefilled();
        ::std::assert_eq!(append_floor(&floor, &mut cmds), Ok(()), "樓梯圖 {:?} 沒有裂縫，應該回傳 Ok", floor);
        let mut expected = prefilled();
        expected.extend(floor.chars().map(|ch| step_word(ch).to_string()));
        ::std::assert_eq!(cmds, expected, "樓梯圖 {:?} 的指令要依序接在原本的指令後面", floor);
    }
}

#[test]
fn crack_at_start_leaves_cmds_untouched() {
    for floor in sound_floors() {
        for crack in CRACK_CHARS {
            let broken = ::std::format!("{}{}", crack, floor);
            let mut cmds = prefilled();
            let result = append_floor(&broken, &mut cmds);
            ::std::assert!(is_crack(&result, 0, crack), "樓梯圖 {:?} 第 0 階裂開，實際 {:?}", broken, result);
            ::std::assert_eq!(cmds, prefilled(), "樓梯圖 {:?} 失敗時，cmds 應該和呼叫前一模一樣", broken);
        }
    }
}

#[test]
fn crack_in_middle_leaves_cmds_untouched() {
    for floor in sound_floors() {
        for cut in 1..floor.len() {
            for crack in CRACK_CHARS {
                let broken = ::std::format!("{}{}{}", &floor[..cut], crack, &floor[cut..]);
                let mut cmds = prefilled();
                let result = append_floor(&broken, &mut cmds);
                ::std::assert!(is_crack(&result, cut, crack), "樓梯圖 {:?} 第 {} 階裂開，實際 {:?}", broken, cut, result);
                ::std::assert_eq!(
                    cmds,
                    prefilled(),
                    "樓梯圖 {:?} 失敗時，前面 {} 階的指令不該留在 cmds 裡",
                    broken,
                    cut
                );
            }
        }
    }
}

#[test]
fn crack_at_end_leaves_cmds_untouched() {
    for floor in sound_floors() {
        for crack in CRACK_CHARS {
            let broken = ::std::format!("{}{}", floor, crack);
            let mut cmds = prefilled();
            let result = append_floor(&broken, &mut cmds);
            ::std::assert!(
                is_crack(&result, floor.len(), crack),
                "樓梯圖 {:?} 最後一階裂開，實際 {:?}",
                broken,
                result
            );
            ::std::assert_eq!(cmds, prefilled(), "樓梯圖 {:?} 失敗時，整層的指令都不該留下", broken);
        }
    }
}

#[test]
fn route_reaches_tower_top() {
    assert_cleared(&level_grid(), &route());
}
