use std::cmp::Reverse;

// 參賽的螃蟹
#[derive(Debug, Clone, PartialEq)]
struct Crab {
    name: String,
    score: u32,
}

fn crab(name: &str, score: u32) -> Crab {
    Crab { name: name.to_string(), score }
}

// 排名次：分數高到低，同分依名字
fn rank(crabs: &mut Vec<Crab>) {
    crabs.sort_by_key(|c| Reverse(c.score));
    crabs.sort_by_key(|c| c.name.clone());
}

fn main() {
    let mut crabs = vec![crab("小石", 5), crab("珍珠", 9), crab("丁丁", 3), crab("海星", 7)];
    rank(&mut crabs);
    for (i, c) in crabs.iter().enumerate() {
        println!("第 {} 名：{}（{} 分）", i + 1, c.name, c.score);
    }
}
