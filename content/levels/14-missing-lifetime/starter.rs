// 擦掉名字前後的青苔（空白）；只有一個參考輸入，編譯器自己推得出回傳值從哪來
fn clean(name: &str) -> &str {
    name.trim()
}

// 古碑上刻著兩個名字，回傳字數較多的那個（一樣多就回傳第一個）
fn longer(a: &str, b: &str) -> &str {
    if a.chars().count() >= b.chars().count() { a } else { b }
}

// 解讀古碑：先擦乾淨，再挑較長的名字
fn read_tablet(left: &str, right: &str) -> String {
    longer(clean(left), clean(right)).to_string()
}

fn main() {
    let left = String::from("  蟹王  ");
    let right = String::from(" 海鷗守衛 ");
    println!("古碑上較長的名字：{}", read_tablet(&left, &right));
}
