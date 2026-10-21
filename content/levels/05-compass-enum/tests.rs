#[test]
fn command_text() {
    ::std::assert_eq!(command(Dir::Up), "MOVE UP");
    ::std::assert_eq!(command(Dir::Down), "MOVE DOWN");
    ::std::assert_eq!(command(Dir::Left), "MOVE LEFT");
    ::std::assert_eq!(command(Dir::Right), "MOVE RIGHT");
}

#[test]
fn delta_matches_direction() {
    ::std::assert_eq!(delta(Dir::Up), (0, -1));
    ::std::assert_eq!(delta(Dir::Down), (0, 1));
    ::std::assert_eq!(delta(Dir::Left), (-1, 0));
    ::std::assert_eq!(delta(Dir::Right), (1, 0));
}

#[test]
fn route_reaches_temple() {
    assert_cleared(&level_grid(), &route());
}
