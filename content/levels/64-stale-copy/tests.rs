// 測試自己排一隊螃蟹：第 i 隻血量上限 6 + i，目前血量 i % 4
fn squad(n: u32) -> Vec<Crab> {
    (0..n).map(|i| Crab { name: ::std::format!("試煉蟹 {}", i + 1), hp: i % 4, max_hp: 6 + i }).collect()
}

#[test]
fn heal_the_two_crabs() {
    let mut crabs = ::std::vec![
        Crab { name: String::from("阿鉗"), hp: 3, max_hp: 10 },
        Crab { name: String::from("小殼"), hp: 5, max_hp: 8 },
    ];
    heal_all(&mut crabs);
    ::std::assert_eq!(crabs[0].hp, 10, "阿鉗的血量應為 10，實際 {}", crabs[0].hp);
    ::std::assert_eq!(crabs[1].hp, 8, "小殼的血量應為 8，實際 {}", crabs[1].hp);
}

#[test]
fn heal_every_crab_in_the_squad() {
    for n in 0..7 {
        let mut crabs = squad(n);
        heal_all(&mut crabs);
        for crab in &crabs {
            ::std::assert_eq!(crab.hp, crab.max_hp, "{} 的血量應為 {}，實際 {}", crab.name, crab.max_hp, crab.hp);
        }
    }
}

#[test]
fn healing_keeps_the_squad() {
    let mut crabs = squad(5);
    heal_all(&mut crabs);
    let names: Vec<&str> = crabs.iter().map(|c| c.name.as_str()).collect();
    ::std::assert_eq!(names, ::std::vec!["試煉蟹 1", "試煉蟹 2", "試煉蟹 3", "試煉蟹 4", "試煉蟹 5"], "治療之後隊伍的螃蟹變了");
    let caps: Vec<u32> = crabs.iter().map(|c| c.max_hp).collect();
    ::std::assert_eq!(caps, ::std::vec![6, 7, 8, 9, 10], "治療不該改動血量上限");
}
