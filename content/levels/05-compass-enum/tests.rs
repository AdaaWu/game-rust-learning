#[test]
fn command_text() {
    assert_eq!(command(Dir::Up), "MOVE UP");
    assert_eq!(command(Dir::Down), "MOVE DOWN");
    assert_eq!(command(Dir::Left), "MOVE LEFT");
    assert_eq!(command(Dir::Right), "MOVE RIGHT");
}

#[test]
fn delta_matches_direction() {
    assert_eq!(delta(Dir::Up), (0, -1));
    assert_eq!(delta(Dir::Down), (0, 1));
    assert_eq!(delta(Dir::Left), (-1, 0));
    assert_eq!(delta(Dir::Right), (1, 0));
}

#[test]
fn route_reaches_temple() {
    assert_cleared(&level_grid(), &route());
}
