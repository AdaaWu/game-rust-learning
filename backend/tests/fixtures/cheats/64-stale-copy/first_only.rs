// 取巧：只治療第一隻螃蟹
// 試煉場的螃蟹：名字、目前血量、血量上限
#[derive(Debug, Clone, PartialEq)]
struct Crab {
    name: String,
    hp: u32,
    max_hp: u32,
}

// 治療所有螃蟹：每一隻的血量補滿到上限
fn heal_all(crabs: &mut Vec<Crab>) {
    if let Some(crab) = crabs.first_mut() {
        crab.hp = crab.max_hp;
    }
}

fn main() {
    let mut crabs = vec![
        Crab { name: String::from("阿鉗"), hp: 3, max_hp: 10 },
        Crab { name: String::from("小殼"), hp: 5, max_hp: 8 },
    ];
    heal_all(&mut crabs);
    for crab in &crabs {
        println!("{} 血量 {}/{}", crab.name, crab.hp, crab.max_hp);
    }
}
