#[test]
fn rare_gem_scores_double() {
    ::std::assert_eq!(score(Tile::Gem(12)), 24, "12 分的稀有寶石鑑定後應該加倍成 24 分");
    ::std::assert_eq!(score(Tile::Gem(10)), 20, "剛好 10 分也算稀有寶石，應該加倍成 20 分");
}

#[test]
fn common_gem_keeps_score() {
    ::std::assert_eq!(score(Tile::Gem(3)), 3, "3 分的一般寶石應該維持 3 分");
    ::std::assert_eq!(score(Tile::Gem(9)), 9, "9 分還不算稀有，應該維持 9 分");
}

#[test]
fn sand_and_door_score_zero() {
    ::std::assert_eq!(score(Tile::Sand), 0, "沙地沒有分數");
    ::std::assert_eq!(score(Tile::Door), 0, "門沒有分數");
}

#[test]
fn door_opens_before_entering() {
    ::std::assert_eq!(
        enter(Tile::Door),
        ::std::vec!["OPEN RIGHT", "MOVE RIGHT"],
        "門是關著的，要先 OPEN RIGHT 才能 MOVE RIGHT"
    );
}

#[test]
fn gem_is_picked_after_entering() {
    ::std::assert_eq!(
        enter(Tile::Gem(7)),
        ::std::vec!["MOVE RIGHT", "PICK"],
        "要先走到寶石那一格再 PICK"
    );
    ::std::assert_eq!(enter(Tile::Sand), ::std::vec!["MOVE RIGHT"], "沙地直接走過去就好");
}

#[test]
fn shore_total_score() {
    let (_, total) = appraise(&SHORE);
    ::std::assert_eq!(total, 32, "長廊的寶石 3 + 12×2 + 5，總分應該是 32");
}

#[test]
fn ferris_clears_treasure_hall() {
    assert_cleared(&level_grid(), &route());
}
