#[test]
fn fresh_ferris_is_not_tired() {
    let ferris = Ferris::new(3);
    ::std::assert!(!ferris.is_tired(), "體力 3 點的 Ferris 還不該喊累");
    ::std::assert!(Ferris::new(0).is_tired(), "體力 0 點的 Ferris 應該已經累了");
}

#[test]
fn step_moves_and_costs_hp() {
    let mut ferris = Ferris::new(3);
    ferris.step(Dir::Right);
    ferris.step(Dir::Down);
    ::std::assert_eq!(ferris.pos, (1, 1), "往右再往下後，Ferris 應該在 (1, 1)");
    ::std::assert_eq!(ferris.hp, 1, "走兩步應該扣 2 點體力，剩 1 點");
    ::std::assert_eq!(ferris.log, ::std::vec!["MOVE RIGHT", "MOVE DOWN"], "狀態欄應該依序記下兩個移動指令");
}

#[test]
fn pick_fills_bag() {
    let mut ferris = Ferris::new(3);
    ferris.pick();
    ferris.pick();
    ::std::assert_eq!(ferris.gems, 2, "撿兩次，背包裡應該有 2 顆寶石");
    ::std::assert_eq!(ferris.log, ::std::vec!["PICK", "PICK"], "狀態欄應該記下兩個 PICK");
    ::std::assert_eq!(ferris.hp, 3, "撿寶石不該消耗體力");
}

#[test]
fn tired_ferris_stays_put() {
    let mut ferris = Ferris::new(1);
    ferris.step(Dir::Right);
    ferris.step(Dir::Down);
    ::std::assert_eq!(ferris.pos, (1, 0), "體力用完後 Ferris 應該停在原地");
    ::std::assert_eq!(ferris.hp, 0, "體力不能扣到 0 以下");
    ::std::assert_eq!(ferris.log, ::std::vec!["MOVE RIGHT"], "累了之後不該再記下移動指令");
}

#[test]
fn ferris_reaches_temple() {
    assert_cleared(&level_grid(), &route());
}
