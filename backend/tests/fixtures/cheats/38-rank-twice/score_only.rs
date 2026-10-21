// 取巧：只排分數，同分的順序沿用出場順序（穩定排序），不比名字
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
    crabs.sort_by(|a, b| b.score.cmp(&a.score));
}

fn main() {
    let mut crabs = vec![crab("小石", 5), crab("珍珠", 9), crab("丁丁", 3), crab("海星", 7)];
    rank(&mut crabs);
    for (i, c) in crabs.iter().enumerate() {
        println!("第 {} 名：{}（{} 分）", i + 1, c.name, c.score);
    }
}
