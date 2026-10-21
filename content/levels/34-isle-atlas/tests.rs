// 測試用的簡易亂數（線性同餘），每次產生不同的地名冊與停靠清單
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

const POOL: [&str; 7] = ["岩蟹灣", "海葵角", "鯨骨灘", "紅樹林", "鹽田島", "霧燈塔", "潮池"];

#[test]
fn atlas_finds_every_isle() {
    for round in 0..10u64 {
        let mut seed = round + 7;
        let count = (round as usize) % POOL.len() + 1;
        let entries: Vec<(&str, (i32, i32))> = POOL[..count]
            .iter()
            .map(|name| (*name, ((next_rand(&mut seed) % 12) as i32, (next_rand(&mut seed) % 12) as i32)))
            .collect();
        let atlas = build_atlas(&entries);
        ::std::assert_eq!(atlas.len(), count, "地名冊有 {} 座島，對照表卻有 {} 筆", count, atlas.len());
        for (name, spot) in &entries {
            ::std::assert_eq!(atlas.get(*name), Some(spot), "查 {} 應該拿到 {:?}", name, spot);
        }
        ::std::assert_eq!(atlas.get("龍蝦島"), None, "地名冊沒有龍蝦島，get 應該回傳 None");
    }
}

#[test]
fn atlas_later_entry_wins() {
    let atlas = build_atlas(&[("潮池", (1, 1)), ("霧燈塔", (2, 2)), ("潮池", (3, 3))]);
    ::std::assert_eq!(atlas.len(), 2, "同一個地名只能佔一筆");
    ::std::assert_eq!(atlas.get("潮池"), Some(&(3, 3)), "同名再 insert 一次，應該以後來的座標為準");
}

#[test]
fn first_visits_skips_seen_and_keeps_order() {
    for round in 0..20u64 {
        let mut seed = round * 31 + 5;
        let len = (next_rand(&mut seed) % 15) as usize;
        let stops: Vec<&str> = (0..len).map(|_| POOL[(next_rand(&mut seed) % 7) as usize]).collect();
        let already = POOL[(round % 7) as usize];
        let mut seen = HashSet::new();
        seen.insert(already.to_string());

        // 用最笨的方法算出預期：一個一個比
        let mut expected: Vec<String> = Vec::new();
        for s in &stops {
            if *s != already && !expected.iter().any(|e| e == s) {
                expected.push(s.to_string());
            }
        }

        let got = first_visits(&stops, &mut seen);
        ::std::assert!(
            !got.iter().any(|g| g == already),
            "{} 已經在 seen 裡（之前去過），這次不該再去",
            already
        );
        ::std::assert_eq!(got, expected, "停靠清單 {:?} 應該只留第一次出現的地名，並保持原本順序", stops);
        for s in &stops {
            ::std::assert!(seen.contains(*s), "走完之後 seen 應該記得 {}", s);
        }
        ::std::assert_eq!(seen.len(), expected.len() + 1, "seen 應該剛好是原本的 1 個加上這次新去的島");
    }
}

#[test]
fn first_visits_nothing_new() {
    let mut seen = HashSet::new();
    ::std::assert!(first_visits(&[], &mut seen).is_empty(), "沒有停靠點就不該去任何島");
    seen.insert("潮池".to_string());
    ::std::assert!(first_visits(&["潮池", "潮池"], &mut seen).is_empty(), "全都去過了，這次不該再去");
    ::std::assert_eq!(seen.len(), 1, "沒有新地名時 seen 不該變多");
}

#[test]
fn locate_skips_unknown_isles() {
    for round in 0..10u64 {
        let mut seed = round * 17 + 3;
        let entries: Vec<(&str, (i32, i32))> = POOL[..4]
            .iter()
            .map(|name| (*name, ((next_rand(&mut seed) % 12) as i32, (next_rand(&mut seed) % 12) as i32)))
            .collect();
        let atlas = build_atlas(&entries);
        let len = (next_rand(&mut seed) % 10) as usize;
        let stops: Vec<String> = (0..len).map(|_| POOL[(next_rand(&mut seed) % 7) as usize].to_string()).collect();
        let expected: Vec<(i32, i32)> = stops
            .iter()
            .filter_map(|s| entries.iter().find(|(n, _)| n == s).map(|(_, p)| *p))
            .collect();
        ::std::assert_eq!(locate(&atlas, &stops), expected, "停靠清單 {:?} 查座標時，地名冊沒有的島要略過、順序不變", stops);
    }
}

#[test]
fn route_visits_every_isle_once() {
    assert_cleared(&level_grid(), &route());
}
