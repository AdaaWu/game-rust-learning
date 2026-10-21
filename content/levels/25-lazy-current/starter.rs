// 統計每片海灘撿到的寶石，回傳總數與工作日誌
fn haul(beaches: &[u32]) -> (u32, Vec<String>) {
    let mut total = 0;
    let mut log = Vec::new();
    beaches.iter().enumerate().map(|(i, gems)| {
        total += gems;
        log.push(format!("第 {} 片海灘：{} 顆", i + 1, gems));
    });
    (total, log)
}

// 收穫超過門檻的海灘編號（從 1 起算）
fn rich_beaches(beaches: &[u32], threshold: u32) -> Vec<usize> {
    beaches
        .iter()
        .enumerate()
        .filter(|(_, gems)| **gems > threshold)
        .map(|(i, _)| i + 1)
        .collect()
}

fn main() {
    let beaches = [3, 5, 7];
    let (total, log) = haul(&beaches);
    for line in log {
        println!("{}", line);
    }
    println!("總共 {} 顆寶石", total);
    println!("豐收的海灘：{:?}", rich_beaches(&beaches, 4));
}
