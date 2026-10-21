// 測試自己產生的期望指令：n 個 word
fn repeat(word: &str, n: usize) -> Vec<String> {
    (0..n).map(|_| word.to_string()).collect()
}

#[test]
fn pick_spell_picks() {
    ::std::assert_eq!(steps!(PICK, 2), repeat("PICK", 2), "steps!(PICK, 2) 應該是兩個 PICK，不是 MOVE PICK");
    ::std::assert_eq!(steps!(PICK, 1), repeat("PICK", 1), "steps!(PICK, 1) 應該是一個 PICK，不是 MOVE PICK");
}

#[test]
fn move_spells_still_move() {
    for n in 0..5 {
        ::std::assert_eq!(steps!(UP, n), repeat("MOVE UP", n), "steps!(UP, {}) 應該是 {} 個 MOVE UP", n, n);
        ::std::assert_eq!(steps!(LEFT, n), repeat("MOVE LEFT", n), "steps!(LEFT, {}) 應該是 {} 個 MOVE LEFT", n, n);
        ::std::assert_eq!(steps!(PICK, n), repeat("PICK", n), "steps!(PICK, {}) 應該是 {} 個 PICK", n, n);
    }
}

#[test]
fn route_forwards_pick() {
    for k in 1..4 {
        let mut expected = repeat("PICK", 1);
        expected.extend(repeat("MOVE LEFT", k));
        expected.extend(repeat("PICK", k));
        ::std::assert_eq!(
            route![PICK 1, LEFT k, PICK k],
            expected,
            "route![PICK 1, LEFT {}, PICK {}] 轉交給 steps! 的 PICK 也要撿寶石",
            k,
            k
        );
    }
}

#[test]
fn spell_route_reaches_exit() {
    assert_cleared(&level_grid(), &route());
}
