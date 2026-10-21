// 測試用的簡易亂數（線性同餘）
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

const GEMS: [&str; 6] = ["珍珠", "海星", "貝殼", "珊瑚", "琥珀", "海玻璃"];
const ISLES: [&str; 4] = ["珊瑚島", "貝殼島", "海星島", "燈塔礁"];

// 用最笨的方法數：一個一個比
fn naive_count(finds: &[&str], gem: &str) -> u32 {
    finds.iter().filter(|f| **f == gem).count() as u32
}

#[test]
fn tally_counts_each_gem() {
    for round in 0..20u64 {
        let mut seed = round * 13 + 1;
        let len = (next_rand(&mut seed) % 20) as usize;
        let finds: Vec<&str> = (0..len).map(|_| GEMS[(next_rand(&mut seed) % 6) as usize]).collect();
        let ledger = tally(&finds);
        let kinds = GEMS.iter().filter(|g| naive_count(&finds, g) > 0).count();
        ::std::assert_eq!(ledger.len(), kinds, "找到 {:?}，帳本應該有 {} 種寶石", finds, kinds);
        for gem in GEMS {
            let want = naive_count(&finds, gem);
            if want == 0 {
                ::std::assert_eq!(ledger.get(gem), None, "沒找到 {} 就不該記在帳本上", gem);
            } else {
                ::std::assert_eq!(ledger.get(gem), Some(&want), "{} 找到 {} 顆，帳本寫錯了", gem, want);
            }
        }
    }
}

#[test]
fn tally_empty_and_single() {
    ::std::assert!(tally(&[]).is_empty(), "什麼都沒找到，帳本應該是空的");
    let ledger = tally(&["琥珀", "琥珀", "琥珀", "琥珀"]);
    ::std::assert_eq!(ledger.len(), 1, "只有一種寶石");
    ::std::assert_eq!(ledger.get("琥珀"), Some(&4), "琥珀找到 4 顆");
}

#[test]
fn group_by_isle_keeps_find_order() {
    for round in 0..20u64 {
        let mut seed = round * 7 + 11;
        let len = (next_rand(&mut seed) % 15) as usize;
        let finds: Vec<(&str, &str)> = (0..len)
            .map(|_| (ISLES[(next_rand(&mut seed) % 4) as usize], GEMS[(next_rand(&mut seed) % 6) as usize]))
            .collect();
        let groups = group_by_isle(&finds);
        for isle in ISLES {
            let want: Vec<String> = finds.iter().filter(|(i, _)| *i == isle).map(|(_, g)| g.to_string()).collect();
            if want.is_empty() {
                ::std::assert_eq!(groups.get(isle), None, "{} 沒有找到寶石，不該出現在分組裡", isle);
            } else {
                ::std::assert_eq!(groups.get(isle), Some(&want), "{} 的寶石要依找到的順序排", isle);
            }
        }
    }
}

#[test]
fn leaderboard_breaks_ties_by_name() {
    let ledger = tally(&["海星", "貝殼", "珍珠", "海星", "珍珠", "珊瑚", "珍珠"]);
    let expected = ::std::vec![
        ("珍珠".to_string(), 3),
        ("海星".to_string(), 2),
        ("珊瑚".to_string(), 1),
        ("貝殼".to_string(), 1),
    ];
    ::std::assert_eq!(leaderboard(&ledger), expected, "數量多的排前面，同數量依名稱排");
    ::std::assert!(leaderboard(&HashMap::new()).is_empty(), "空帳本的排行榜應該是空的");
}

#[test]
fn leaderboard_same_on_every_fresh_map() {
    let rows: [(&str, u32); 6] = [("琥珀", 2), ("珍珠", 5), ("海玻璃", 2), ("貝殼", 2), ("珊瑚", 7), ("海星", 5)];
    let expected: Vec<(String, u32)> = [("珊瑚", 7), ("海星", 5), ("珍珠", 5), ("海玻璃", 2), ("琥珀", 2), ("貝殼", 2)]
        .iter()
        .map(|(n, c)| (n.to_string(), *c))
        .collect();
    // 每一次都建一個新的 HashMap，插入順序也輪替；走訪順序會跟著變
    for round in 0..20 {
        let mut ledger = HashMap::new();
        for i in 0..rows.len() {
            let (name, count) = rows[(i + round) % rows.len()];
            ledger.insert(name.to_string(), count);
        }
        ::std::assert_eq!(leaderboard(&ledger), expected, "第 {} 本帳的排行榜不一樣：排行榜不能依賴 HashMap 的走訪順序", round + 1);
    }
}

#[test]
fn leaderboard_random_ledgers_sorted() {
    for round in 0..20u64 {
        let mut seed = round * 29 + 3;
        let mut ledger = HashMap::new();
        for gem in GEMS {
            if next_rand(&mut seed) % 4 != 0 {
                ledger.insert(gem.to_string(), (next_rand(&mut seed) % 3) as u32 + 1);
            }
        }
        let board = leaderboard(&ledger);
        ::std::assert_eq!(board.len(), ledger.len(), "排行榜要列出帳本上的每一種寶石");
        for (name, count) in &board {
            ::std::assert_eq!(ledger.get(name), Some(count), "排行榜上 {} 的數量和帳本不同", name);
        }
        for pair in board.windows(2) {
            let ok = pair[0].1 > pair[1].1 || (pair[0].1 == pair[1].1 && pair[0].0 < pair[1].0);
            ::std::assert!(ok, "{:?} 不該排在 {:?} 前面", pair[0], pair[1]);
        }
    }
}
