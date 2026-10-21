// 先借海鷗數一數，放入紅寶石，再借海鷗數一次
fn trip() -> (usize, usize, Vec<String>) {
    let mut bag = vec![String::from("藍寶石"), String::from("貝殼")];
    let before = 0; // TODO 借給海鷗數：gull_count(...)
    // TODO 放入「紅寶石」：put_in(...)
    let after = 0; // TODO 再借給海鷗數一次
    (before, after, bag)
}

// 海鷗只看不拿：數出名稱以「寶石」結尾的有幾樣
fn gull_count(bag: &Vec<String>) -> usize {
    let mut count = 0;
    for item in bag {
        todo!("item 是 &String，名稱以「寶石」結尾就把 count 加 1")
    }
    count
}

// Ferris 把新寶物放進背包
fn put_in(bag: &mut Vec<String>, item: &str) {
    todo!("把 item 轉成 String 放進 bag")
}

fn main() {
    let (before, after, bag) = trip();
    println!("海鷗數到 {} 顆，放入後變 {} 顆，背包：{:?}", before, after, bag);
}
