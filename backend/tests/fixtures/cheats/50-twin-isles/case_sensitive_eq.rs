// 取巧：Hash 與 Eq 都改用 derive（分大小寫），chart 自己逐一比對名稱去重
use std::collections::HashSet;

// 海圖上的一座島：名稱不分大小寫
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Isle {
    name: String,
}

impl Isle {
    fn new(name: &str) -> Isle {
        Isle { name: name.to_string() }
    }
}

// 把海鷗回報的島名記到海圖上，同一座島只記一次
fn chart(sightings: &[&str]) -> HashSet<Isle> {
    let mut isles = HashSet::new();
    for name in sightings {
        if !isles.iter().any(|i: &Isle| i.name.eq_ignore_ascii_case(name)) {
            isles.insert(Isle::new(name));
        }
    }
    isles
}

fn main() {
    let isles = chart(&["Coral", "Reef", "coral", "CORAL"]);
    println!("海圖上有 {} 座島", isles.len());
}
