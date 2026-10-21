use std::collections::HashMap;

// 數每種寶石各找到幾顆
fn tally(finds: &[&str]) -> HashMap<String, u32> {
    let mut ledger = HashMap::new();
    for gem in finds {
        let count = ledger.entry(gem.to_string()).or_insert(0);
        *count += 1;
    }
    ledger
}

// 依島嶼分組：島名 → 依找到順序排列的寶石
fn group_by_isle(finds: &[(&str, &str)]) -> HashMap<String, Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for (isle, gem) in finds {
        groups.entry(isle.to_string()).or_default().push(gem.to_string());
    }
    groups
}

// 排行榜：數量多到少，同數量依名稱
fn leaderboard(ledger: &HashMap<String, u32>) -> Vec<(String, u32)> {
    let mut rows: Vec<(String, u32)> = ledger.iter().map(|(k, v)| (k.clone(), *v)).collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows
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
