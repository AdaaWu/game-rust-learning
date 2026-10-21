// 測試用的簡易亂數（線性同餘）
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

const STALLS: [&str; 9] = ["1號攤", "2號攤", "3號攤", "4號攤", "5號攤", "6號攤", "7號攤", "8號攤", "9號攤"];

fn stock_of(rows: &[(&str, u32)]) -> HashMap<String, u32> {
    rows.iter().map(|(s, n)| (s.to_string(), *n)).collect()
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn trade_line_parses() {
    let got = "3號攤 出 2".parse::<Trade>();
    ::std::assert!(
        ::std::matches!(&got, Ok(Trade { stall, flow: Flow::Out, qty: 2 }) if stall == "3號攤"),
        "「3號攤 出 2」應該解析成 3號攤賣出 2 件，實際 {:?}",
        got
    );
    let got = "8號攤 進 10".parse::<Trade>();
    ::std::assert!(
        ::std::matches!(&got, Ok(Trade { stall, flow: Flow::In, qty: 10 }) if stall == "8號攤"),
        "「8號攤 進 10」應該解析成 8號攤進貨 10 件，實際 {:?}",
        got
    );
    for bad in ["3號攤 出", "3號攤 借 2", "3號攤 進 兩", "3號攤 進 -1", "3號攤 進 2 件"] {
        let got = bad.parse::<Trade>();
        ::std::assert!(
            ::std::matches!(&got, Err(TradeError::BadLine(line)) if line == bad),
            "看不懂的紀錄「{}」要回報 BadLine，實際 {:?}",
            bad,
            got
        );
    }
}

#[test]
fn parse_trades_skips_blank_and_stops_at_bad_line() {
    let ok = parse_trades("\n1號攤 進 3\n\n2號攤 出 0\n").expect("只有空白行與正確紀錄，應該解析成功");
    ::std::assert_eq!(ok.len(), 2, "空白行要略過，只留 2 筆交易");
    let got = parse_trades("1號攤 進 3\n1號攤 偷 1\n");
    ::std::assert!(
        ::std::matches!(&got, Err(TradeError::BadLine(line)) if line == "1號攤 偷 1"),
        "遇到看不懂的紀錄要回報是哪一行，實際 {:?}",
        got
    );
}

#[test]
fn stock_single_trade() {
    let trades = parse_trades("7號攤 進 4").unwrap();
    ::std::assert_eq!(stock(&trades).get("7號攤"), Some(&4), "7號攤只進貨 4 件，收市應該剩 4 件");
    let trades = parse_trades("7號攤 進 4\n7號攤 出 4").unwrap();
    ::std::assert_eq!(stock(&trades).get("7號攤"), Some(&0), "7號攤進 4 件又賣 4 件，應該售完（剩 0 件）");
}

#[test]
fn stock_counts_every_stall() {
    for round in 0..20u64 {
        let mut seed = round * 43 + 1;
        let mut held = [0u32; 9];
        let mut log = String::new();
        for _ in 0..(next_rand(&mut seed) % 25) {
            let k = (next_rand(&mut seed) % 9) as usize;
            if held[k] > 0 && next_rand(&mut seed) % 2 == 0 {
                let qty = (next_rand(&mut seed) % held[k] as u64) as u32 + 1;
                held[k] -= qty;
                log.push_str(&::std::format!("{} 出 {}\n", STALLS[k], qty));
            } else {
                let qty = (next_rand(&mut seed) % 5) as u32 + 1;
                held[k] += qty;
                log.push_str(&::std::format!("{} 進 {}\n", STALLS[k], qty));
            }
        }
        let left = stock(&parse_trades(&log).unwrap());
        for k in 0..9 {
            if log.contains(STALLS[k]) {
                ::std::assert_eq!(
                    left.get(STALLS[k]),
                    Some(&held[k]),
                    "{} 收市應該剩 {} 件，帳上卻不對",
                    STALLS[k],
                    held[k]
                );
            } else {
                ::std::assert_eq!(left.get(STALLS[k]), None, "{} 今天沒有交易，不該出現在帳上", STALLS[k]);
            }
        }
    }
}

#[test]
fn visit_order_same_on_every_fresh_map() {
    let rows: [(&str, u32); 6] = [("6號攤", 2), ("2號攤", 1), ("9號攤", 3), ("1號攤", 0), ("4號攤", 5), ("7號攤", 1)];
    let want = names(&["2號攤", "4號攤", "6號攤", "7號攤", "9號攤"]);
    // 每一次都建一個新的 HashMap，插入順序也輪替；走訪順序會跟著變
    for round in 0..20 {
        let mut map = HashMap::new();
        for i in 0..rows.len() {
            let (s, n) = rows[(i + round) % rows.len()];
            map.insert(s.to_string(), n);
        }
        ::std::assert_eq!(visit_order(&map), want, "第 {} 次重建帳本後，拜訪順序不一樣：要照攤位名稱排", round + 1);
    }
}

#[test]
fn visit_order_drops_adjacent_sold_out() {
    let map = stock_of(&[("1號攤", 2), ("2號攤", 0), ("3號攤", 0), ("4號攤", 0), ("5號攤", 1)]);
    let got = visit_order(&map);
    for sold in ["2號攤", "3號攤", "4號攤"] {
        ::std::assert!(!got.iter().any(|s| s == sold), "{} 已經售完，不該還在拜訪清單上：{:?}", sold, got);
    }
    ::std::assert_eq!(got, names(&["1號攤", "5號攤"]), "只剩 1號攤和 5號攤有貨");
    let none = stock_of(&[("8號攤", 0), ("9號攤", 0)]);
    ::std::assert!(visit_order(&none).is_empty(), "全部售完時不用拜訪任何攤位");
}

#[test]
fn visit_order_random_stock() {
    for round in 0..20u64 {
        let mut seed = round * 59 + 8;
        let mut rows: Vec<(&str, u32)> = Vec::new();
        for s in STALLS {
            if next_rand(&mut seed) % 3 != 0 {
                rows.push((s, (next_rand(&mut seed) % 3) as u32));
            }
        }
        // STALLS 本來就照名稱排好，篩掉售完的就是預期
        let want: Vec<String> = rows.iter().filter(|(_, n)| *n > 0).map(|(s, _)| s.to_string()).collect();
        ::std::assert_eq!(visit_order(&stock_of(&rows)), want, "帳本 {:?} 的拜訪順序不對", rows);
    }
}

#[test]
fn route_clears_the_market() {
    assert_cleared(&level_grid(), &route());
}
