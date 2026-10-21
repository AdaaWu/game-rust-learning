#[test]
fn walk_repeats_command() {
    assert_eq!(
        walk("RIGHT", 3),
        vec!["MOVE RIGHT", "MOVE RIGHT", "MOVE RIGHT"],
        "walk(\"RIGHT\", 3) 應該是三個 MOVE RIGHT"
    );
    assert_eq!(walk("UP", 1), vec!["MOVE UP"], "walk(\"UP\", 1) 應該只有一個 MOVE UP");
}

#[test]
fn walk_zero_steps_is_empty() {
    assert!(walk("LEFT", 0).is_empty(), "走 0 步時 Ferris 應該原地不動，清單要是空的");
}

#[test]
fn patrol_rows_counts_every_other_row() {
    assert_eq!(patrol_rows(5), 3, "高度 5 的海岸要巡第 0、2、4 列，共 3 列");
    assert_eq!(patrol_rows(4), 2, "高度 4 的海岸只巡得到第 0、2 列，共 2 列");
    assert_eq!(patrol_rows(1), 1, "只有 1 列的海岸也要巡 1 列");
    assert_eq!(patrol_rows(0), 0, "沒有海岸就不用巡邏");
}

#[test]
fn ferris_patrols_whole_shore() {
    assert_cleared(&level_grid(), &route());
}
