#[test]
fn walk_into_any_bag() {
    let mut pack: Vec<String> = Vec::new();
    walk!(pack, LEFT);
    walk!(pack, UP);
    ::std::assert_eq!(pack, ::std::vec!["MOVE LEFT", "MOVE UP"], "walk!(pack, ...) 要把指令記進名叫 pack 的行囊");
}

#[test]
fn grab_into_any_bag() {
    let mut sack: Vec<String> = ::std::vec!["MOVE DOWN".to_string()];
    grab!(sack);
    ::std::assert_eq!(sack, ::std::vec!["MOVE DOWN", "PICK"], "grab!(sack) 要把 PICK 接在 sack 的最後面");
}

#[test]
fn two_bags_stay_apart() {
    for rounds in 0..6 {
        let mut left: Vec<String> = Vec::new();
        let mut right: Vec<String> = Vec::new();
        for i in 0..rounds {
            if i % 2 == 0 {
                walk!(left, RIGHT);
            } else {
                walk!(right, DOWN);
                grab!(right);
            }
        }
        let evens = (rounds + 1) / 2;
        let odds = rounds / 2;
        ::std::assert_eq!(left.len(), evens, "{} 輪後 left 行囊應該有 {} 道指令", rounds, evens);
        ::std::assert_eq!(right.len(), odds * 2, "{} 輪後 right 行囊應該有 {} 道指令", rounds, odds * 2);
        ::std::assert!(left.iter().all(|c| c == "MOVE RIGHT"), "left 行囊混進了別的指令：兩個行囊要各記各的");
    }
}

#[test]
fn spell_route_reaches_exit() {
    assert_cleared(&level_grid(), &route());
}
