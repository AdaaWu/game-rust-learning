// 先借海鷗數一數，放入紅寶石，再借海鷗數一次
fn trip() -> (usize, usize, Vec<String>) {
    let mut bag = vec![String::from("藍寶石"), String::from("貝殼")];
    let before = gull_count(&bag);
    put_in(&mut bag, "紅寶石");
    let after = gull_count(&bag);
    (before, after, bag)
}

// 海鷗只看不拿：數出名稱以「寶石」結尾的有幾樣
fn gull_count(bag: &Vec<String>) -> usize {
    let mut count = 0;
    for item in bag {
        if item.ends_with("寶石") { count += 1; }
    }
    count
}

// Ferris 把新寶物放進背包
fn put_in(bag: &mut Vec<String>, item: &str) {
    bag.push(item.to_string());
}

fn main() {
    let (before, after, bag) = trip();
    println!("海鷗數到 {} 顆，放入後變 {} 顆，背包：{:?}", before, after, bag);
}
