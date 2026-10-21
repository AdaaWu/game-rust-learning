fn door_of(floor: u32, lock: u32, key: u32, spare: u32) -> Door {
    Door { floor, lock, key, spare }
}

// 測試自己算出一座塔該有的指令
fn expected_route(spec: &[(i32, u32)]) -> Vec<String> {
    let mut cmds = Vec::new();
    for &(walk, _) in spec {
        let side = if walk > 0 { "MOVE RIGHT" } else { "MOVE LEFT" };
        for _ in 0..walk.abs() {
            cmds.push(side.to_string());
        }
        cmds.push("OPEN UP".to_string());
        cmds.push("MOVE UP".to_string());
        cmds.push("MOVE UP".to_string());
    }
    cmds
}

fn prefilled() -> Vec<String> {
    ::std::vec!["MOVE RIGHT".to_string(), "PICK".to_string()]
}

#[test]
fn main_key_opens_door() {
    for lock in [1, 4, 7, 12] {
        let mut cmds = prefilled();
        let result = unlock(&door_of(3, lock, lock, lock + 1), &mut cmds);
        ::std::assert_eq!(result, Ok(()), "主鑰匙對的時候應該打得開（鎖 {}）", lock);
        let mut expected = prefilled();
        expected.push("OPEN UP".to_string());
        ::std::assert_eq!(cmds, expected, "門打開後只該多記一個 OPEN UP");
    }
}

#[test]
fn spare_key_opens_when_main_is_wrong() {
    for (lock, key) in [(4, 9), (2, 3), (8, 1)] {
        let mut cmds = prefilled();
        let result = unlock(&door_of(2, lock, key, lock), &mut cmds);
        ::std::assert_eq!(result, Ok(()), "主鑰匙錯、備用鑰匙對，門應該打得開");
        let mut expected = prefilled();
        expected.push("OPEN UP".to_string());
        ::std::assert_eq!(cmds, expected, "備用鑰匙打開門後要記下 OPEN UP");
    }
}

#[test]
fn both_keys_wrong_reports_locked() {
    for floor in [1, 2, 5, 9] {
        let mut cmds = prefilled();
        let result = unlock(&door_of(floor, 6, 1, 2), &mut cmds);
        ::std::assert!(
            ::std::matches!(result, Err(TowerError::Locked { floor: f }) if f == floor),
            "兩把鑰匙都錯，應該回報 Locked（第 {} 層），實際 {:?}",
            floor,
            result
        );
        ::std::assert_eq!(cmds, prefilled(), "門沒打開就不該記下 OPEN UP");
    }
}

#[test]
fn climb_generated_towers() {
    let mut seed: u32 = 43;
    for round in 0..30 {
        let count = round % 4 + 1;
        let mut spec = Vec::new();
        let mut floors = Vec::new();
        for f in 0..count {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let walk = (seed >> 8) as i32 % 7 - 3;
            let lock = (seed >> 20) % 9 + 1;
            let use_spare = (seed >> 12) % 2 == 0;
            let (key, spare) = if use_spare { (lock + 1, lock) } else { (lock, lock + 2) };
            spec.push((walk, lock));
            floors.push(Floor { walk, door: door_of(f as u32 + 1, lock, key, spare) });
        }
        ::std::assert_eq!(climb(&floors), Ok(expected_route(&spec)), "每層門都打得開時，路線不對：{:?}", spec);

        let jammed = (round % count) as usize;
        floors[jammed].door.key = 99;
        floors[jammed].door.spare = 98;
        let got = climb(&floors);
        ::std::assert!(
            ::std::matches!(got, Err(TowerError::Locked { floor }) if floor == jammed as u32 + 1),
            "第 {} 層兩把鑰匙都錯，climb 應該停下來回報，實際 {:?}",
            jammed + 1,
            got
        );
    }
}

#[test]
fn route_reaches_tower_top() {
    let cmds = climb(&tower_floors()).expect("這座塔每一層都有鑰匙打得開");
    assert_cleared(&level_grid(), &cmds);
}
