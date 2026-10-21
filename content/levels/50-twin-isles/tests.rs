use std::collections::HashMap;
use std::hash::BuildHasher;

// 測試自己產生的大小寫變化：同一個名字的每個字母各自大寫或小寫
fn variants(name: &str, seed: u32) -> Vec<String> {
    let mut s = seed;
    (0..6)
        .map(|_| {
            name.chars()
                .map(|c| {
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                    if (s >> 16) % 2 == 0 { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() }
                })
                .collect()
        })
        .collect()
}

#[test]
fn name_keeps_original_case() {
    ::std::assert_eq!(Isle::new("Coral").name, "Coral", "島名要照海鷗回報的樣子保存，不能改大小寫");
    ::std::assert_eq!(Isle::new("mAnGo Reef").name, "mAnGo Reef", "島名要照海鷗回報的樣子保存，不能改大小寫");
}

#[test]
fn equal_ignores_case() {
    ::std::assert!(Isle::new("A") == Isle::new("a"), "A 和 a 是同一座島");
    ::std::assert!(Isle::new("Coral") == Isle::new("cORAL"), "Coral 和 cORAL 是同一座島");
    ::std::assert!(Isle::new("Coral") != Isle::new("Coralline"), "Coral 和 Coralline 是不同的島");
}

#[test]
fn equal_isles_hash_the_same() {
    let hasher = std::collections::hash_map::RandomState::new();
    for (i, name) in ["Coral", "Reef", "Turtle Bay", "x"].iter().enumerate() {
        let first = Isle::new(name);
        for v in variants(name, i as u32 + 7) {
            let other = Isle::new(&v);
            ::std::assert!(first == other, "{} 和 {} 應該相等", name, v);
            ::std::assert_eq!(
                hasher.hash_one(&first),
                hasher.hash_one(&other),
                "{} 和 {} 相等，雜湊卻不同：違反 Hash 與 Eq 的契約",
                name,
                v
            );
        }
    }
}

#[test]
fn chart_counts_each_isle_once() {
    let isles = chart(&["Coral", "Reef", "coral", "CORAL"]);
    ::std::assert_eq!(isles.len(), 2, "應該只有 2 座島，實際 {} 座", isles.len());
    ::std::assert!(isles.contains(&Isle::new("cOrAl")), "海圖上找不到 cOrAl，明明記過 Coral");
}

#[test]
fn chart_with_many_spellings() {
    let names = ["Coral", "Reef", "Turtle Bay", "Mango", "Kelp Forest"];
    for round in 0..20u32 {
        let mut seen: Vec<String> = Vec::new();
        for (i, name) in names.iter().enumerate().take(round as usize % 5 + 1) {
            seen.extend(variants(name, round * 31 + i as u32));
        }
        let refs: Vec<&str> = seen.iter().map(|s| s.as_str()).collect();
        let isles = chart(&refs);
        let expected = round as usize % 5 + 1;
        ::std::assert_eq!(isles.len(), expected, "應該只有 {} 座島，實際 {} 座", expected, isles.len());
    }
}

#[test]
fn hashmap_finds_isle_by_any_case() {
    let mut visits: HashMap<Isle, u32> = HashMap::new();
    for name in ["Coral", "REEF", "coral", "Coral", "reef", "cORAL"] {
        *visits.entry(Isle::new(name)).or_insert(0) += 1;
    }
    ::std::assert_eq!(visits.len(), 2, "拜訪紀錄應該只有 2 座島，實際 {} 座", visits.len());
    ::std::assert_eq!(visits.get(&Isle::new("CORAL")), Some(&4), "用 CORAL 查詢，應該查到 Coral 被拜訪 4 次");
    ::std::assert_eq!(visits.get(&Isle::new("Reef")), Some(&2), "用 Reef 查詢，應該查到 REEF 被拜訪 2 次");
}
