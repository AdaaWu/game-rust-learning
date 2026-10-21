#[test]
fn straight_commands_unchanged() {
    ::std::assert_eq!(commands(Dir::Up), ::std::vec!["MOVE UP"], "往上仍然是一個 MOVE UP");
    ::std::assert_eq!(commands(Dir::Right), ::std::vec!["MOVE RIGHT"], "往右仍然是一個 MOVE RIGHT");
}

#[test]
fn diagonal_splits_into_two_moves() {
    ::std::assert_eq!(
        commands(Dir::DownRight),
        ::std::vec!["MOVE DOWN", "MOVE RIGHT"],
        "斜角要拆成先 MOVE DOWN 再 MOVE RIGHT，Ferris 才不會撞上階梯的牆"
    );
}

#[test]
fn diagonal_delta() {
    ::std::assert_eq!(delta(Dir::DownRight), (1, 1), "往右下一格，x 和 y 都要加 1");
    ::std::assert_eq!(delta(Dir::Left), (-1, 0), "往左的位移應該還是 (-1, 0)");
}

#[test]
fn diagonal_costs_two() {
    ::std::assert_eq!(
        cost(Dir::DownRight),
        2,
        "斜角等於走兩步，應該花 2 點體力；cost 的 _ 分支把新方向當成 1 了"
    );
    ::std::assert_eq!(cost(Dir::Up), 2, "爬坡仍然花 2 點體力");
    ::std::assert_eq!(cost(Dir::Down), 1, "往下仍然花 1 點體力");
}

#[test]
fn ferris_descends_the_stairs() {
    assert_cleared(&level_grid(), &route());
}
