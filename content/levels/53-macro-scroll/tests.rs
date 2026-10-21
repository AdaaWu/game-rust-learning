// 測試自己產生的期望指令：n 個 word
fn repeat(word: &str, n: usize) -> Vec<String> {
    (0..n).map(|_| word.to_string()).collect()
}

#[test]
fn steps_moves_in_any_direction() {
    for n in 0..6 {
        ::std::assert_eq!(steps!(LEFT, n), repeat("MOVE LEFT", n), "steps!(LEFT, {}) 應該是 {} 個 MOVE LEFT", n, n);
        ::std::assert_eq!(steps!(UP, n), repeat("MOVE UP", n), "steps!(UP, {}) 應該是 {} 個 MOVE UP", n, n);
    }
}

#[test]
fn steps_zero_is_empty() {
    ::std::assert!(steps!(DOWN, 0).is_empty(), "steps!(DOWN, 0) 不該有任何指令");
    ::std::assert!(steps!(PICK, 0).is_empty(), "steps!(PICK, 0) 不該有任何指令");
}

#[test]
fn steps_pick_is_not_a_move() {
    ::std::assert_eq!(steps!(PICK, 2), repeat("PICK", 2), "steps!(PICK, 2) 應該是兩個 PICK，不是 MOVE PICK");
}

#[test]
fn steps_accepts_variables_and_expressions() {
    let k = 2;
    ::std::assert_eq!(steps!(UP, k), repeat("MOVE UP", 2), "steps!(UP, k) 要能接受變數");
    ::std::assert_eq!(steps!(RIGHT, k + 1), repeat("MOVE RIGHT", 3), "steps!(RIGHT, k + 1) 要能接受運算式");
}

#[test]
fn route_expands_each_group_in_order() {
    let mut expected = repeat("MOVE LEFT", 3);
    expected.extend(repeat("PICK", 1));
    expected.extend(repeat("MOVE UP", 2));
    ::std::assert_eq!(route![LEFT 3, PICK 1, UP 2], expected, "route![LEFT 3, PICK 1, UP 2] 要依序展開每一段");
}

#[test]
fn route_works_with_runtime_counts() {
    for a in 0..4 {
        for b in 0..3 {
            let mut expected = repeat("MOVE DOWN", a);
            expected.extend(repeat("PICK", b));
            expected.extend(repeat("MOVE RIGHT", a + b));
            ::std::assert_eq!(
                route![DOWN a, PICK b, RIGHT a + b],
                expected,
                "route![DOWN {}, PICK {}, RIGHT {}] 展開的指令不對",
                a,
                b,
                a + b
            );
        }
    }
}

#[test]
fn route_with_single_group() {
    ::std::assert_eq!(route![RIGHT 1], repeat("MOVE RIGHT", 1), "只有一段咒語時也要正常展開");
    ::std::assert_eq!(route![PICK 3], repeat("PICK", 3), "route![PICK 3] 應該是三個 PICK");
}

#[test]
fn scroll_route_reaches_exit() {
    assert_cleared(&level_grid(), &route());
}
