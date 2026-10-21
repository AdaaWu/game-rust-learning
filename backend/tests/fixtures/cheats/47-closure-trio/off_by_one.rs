// 取巧：用 0..=n 多捏一下，關卡的 main 只印出長度不會發現
// Ferris 的三種鉗法：看（Fn）、捏（FnMut）、交出去（FnOnce）

// 重複做 n 次同一個動作
fn repeat<F: FnMut()>(n: u32, mut f: F) {
    for _ in 0..=n {
        f();
    }
}

// 把東西交出去：f 只會被呼叫一次
fn deliver<F: FnOnce() -> String>(f: F) -> String {
    f()
}

fn main() {
    let shell = String::from("貝殼");
    repeat(2, || println!("看一眼 {}", shell));

    let mut log: Vec<String> = Vec::new();
    repeat(3, || log.push("捏一下".to_string()));
    println!("捏了 {} 下：{:?}", log.len(), log);

    let gem = String::from("珍珠");
    let got = deliver(move || gem);
    println!("交出 {}", got);
}
