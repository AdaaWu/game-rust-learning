// 取巧：bound 改對了，但 repeat 只捏一下，假設呼叫端不在乎次數
// Ferris 的三種鉗法：看（Fn）、捏（FnMut）、交出去（FnOnce）

// 重複做 n 次同一個動作
fn repeat<F: FnMut()>(n: u32, mut f: F) {
    if n > 0 {
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
