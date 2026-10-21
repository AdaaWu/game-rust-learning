// 取巧：Hash 不修，建構時就把島名轉成小寫，讓 derive(Hash) 剛好一致
use std::collections::HashSet;

// 海圖上的一座島：名稱不分大小寫
#[derive(Debug, Clone, Hash)]
struct Isle {
    name: String,
}

impl Isle {
    fn new(name: &str) -> Isle {
        Isle { name: name.to_lowercase() }
    }
}

// Coral、CORAL、coral 都是同一座島
impl PartialEq for Isle {
    fn eq(&self, other: &Isle) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}

impl Eq for Isle {}

// 把海鷗回報的島名記到海圖上，同一座島只記一次
fn chart(sightings: &[&str]) -> HashSet<Isle> {
    let mut isles = HashSet::new();
    for name in sightings {
        isles.insert(Isle::new(name));
    }
    isles
}

fn main() {
    let isles = chart(&["Coral", "Reef", "coral", "CORAL"]);
    println!("海圖上有 {} 座島", isles.len());
}
