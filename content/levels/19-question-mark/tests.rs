// 把測試用的字元地圖轉成 Grid
fn grid_from(map: &[&str]) -> Grid {
    let mut grid = Grid {
        width: map[0].chars().count() as i32,
        height: map.len() as i32,
        start: (0, 0),
        goal: (0, 0),
        walls: vec![],
        gems: vec![],
        doors: vec![],
    };
    for (y, line) in map.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let p = (x as i32, y as i32);
            match ch {
                'F' => grid.start = p,
                'E' => grid.goal = p,
                'G' => grid.gems.push(p),
                _ => {}
            }
        }
    }
    grid
}

#[test]
fn gem_down_right() {
    let map = ["F....", ".....", "...GE"];
    let cmds = plan(&map).expect("有寶石時應該回傳路線");
    assert_cleared(&grid_from(&map), &cmds);
}

#[test]
fn gem_up_left() {
    let map = ["EG...", ".....", "....F"];
    let cmds = plan(&map).expect("有寶石時應該回傳路線");
    assert_cleared(&grid_from(&map), &cmds);
}

#[test]
fn island_without_gem() {
    let map = ["F....", "....E"];
    assert!(plan(&map).is_none(), "沒有寶石時應該回傳 None");
}

#[test]
fn island_without_ferris() {
    let map = ["....G", "E...."];
    assert!(plan(&map).is_none(), "地圖上沒有 Ferris 時應該回傳 None");
}
