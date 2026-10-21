// 測試用的簡易亂數（線性同餘）
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

const NAMES: [&str; 8] = ["阿寶", "小石", "珍珠", "丁丁", "海星", "貝貝", "大螯", "泡泡"];

fn crabs_of(rows: &[(&str, u32)]) -> Vec<Crab> {
    rows.iter().map(|(n, s)| Crab { name: n.to_string(), score: *s }).collect()
}

// 逐名次比對名字與分數，第一個不一樣的名次直接講出來
fn check(mut crabs: Vec<Crab>) {
    let mut want: Vec<(String, u32)> = crabs.iter().map(|c| (c.name.clone(), c.score)).collect();
    want.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rank(&mut crabs);
    ::std::assert_eq!(crabs.len(), want.len(), "排名前後螃蟹數量不一樣");
    for (i, (got, exp)) in crabs.iter().zip(&want).enumerate() {
        ::std::assert!(
            got.name == exp.0 && got.score == exp.1,
            "第 {} 名應該是 {} 分的{}，實際是 {} 分的{}",
            i + 1,
            exp.1,
            exp.0,
            got.score,
            got.name
        );
    }
}

#[test]
fn sample_contest() {
    check(crabs_of(&[("小石", 5), ("珍珠", 9), ("丁丁", 3), ("海星", 7)]));
}

#[test]
fn ties_sorted_by_name() {
    check(crabs_of(&[("貝貝", 4), ("阿寶", 8), ("小石", 4), ("大螯", 8), ("丁丁", 4)]));
}

#[test]
fn everyone_tied() {
    check(crabs_of(&[("海星", 6), ("丁丁", 6), ("珍珠", 6), ("小石", 6)]));
}

#[test]
fn single_and_empty() {
    check(crabs_of(&[("泡泡", 1)]));
    check(Vec::new());
}

#[test]
fn random_contests() {
    for round in 0..30u64 {
        let mut seed = round * 37 + 2;
        let mut names = NAMES.to_vec();
        // 洗牌：每一輪螃蟹出場順序都不同
        for i in (1..names.len()).rev() {
            let j = (next_rand(&mut seed) % (i as u64 + 1)) as usize;
            names.swap(i, j);
        }
        let count = (next_rand(&mut seed) % 8) as usize + 1;
        let rows: Vec<(&str, u32)> = names[..count].iter().map(|n| (*n, (next_rand(&mut seed) % 4) as u32)).collect();
        check(crabs_of(&rows));
    }
}
