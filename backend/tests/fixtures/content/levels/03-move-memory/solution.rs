// 計算背包裡有幾樣東西
fn count_items(bag: Vec<String>) -> usize {
    bag.len()
}

// 回傳 (數量, 第一樣東西)
fn report() -> (usize, String) {
    let bag = vec![String::from("珍珠"), String::from("貝殼"), String::from("海星")];
    let first = bag[0].clone();
    let n = count_items(bag);
    (n, first)
}

fn main() {
    let (n, first) = report();
    println!("背包有 {} 樣，第一樣是 {}", n, first);
}
