use std::collections::HashMap;

// 數每種寶石各找到幾顆
fn tally(finds: &[&str]) -> HashMap<String, u32> {
    todo!("用 entry 一次完成「沒有就建、有就加一」")
}

// 依島嶼分組：島名 → 依找到順序排列的寶石
fn group_by_isle(finds: &[(&str, &str)]) -> HashMap<String, Vec<String>> {
    todo!("entry(島名) 拿到那座島的 Vec 再 push")
}

// 排行榜：數量多到少，同數量依名稱
fn leaderboard(ledger: &HashMap<String, u32>) -> Vec<(String, u32)> {
    todo!("HashMap 沒有順序：先轉成 Vec 再排序")
}

fn main() {
    let finds = ["珍珠", "海星", "珍珠", "貝殼", "海星", "珍珠"];
    let ledger = tally(&finds);
    for (gem, count) in leaderboard(&ledger) {
        println!("{}：{} 顆", gem, count);
    }
    let by_isle = group_by_isle(&[("珊瑚島", "珍珠"), ("貝殼島", "海星"), ("珊瑚島", "貝殼")]);
    println!("珊瑚島：{:?}", by_isle.get("珊瑚島"));
}
