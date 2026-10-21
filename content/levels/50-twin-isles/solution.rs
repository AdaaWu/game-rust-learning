use std::collections::HashSet;
use std::hash::{Hash, Hasher};

// 海圖上的一座島：名稱不分大小寫
#[derive(Debug, Clone)]
struct Isle {
    name: String,
}

impl Isle {
    fn new(name: &str) -> Isle {
        Isle { name: name.to_string() }
    }
}

// Coral、CORAL、coral 都是同一座島
impl PartialEq for Isle {
    fn eq(&self, other: &Isle) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}

impl Eq for Isle {}

// 相等的島必須有相同的雜湊：和 eq 一樣不分大小寫
impl Hash for Isle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.to_ascii_lowercase().hash(state);
    }
}

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
