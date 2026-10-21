use std::collections::HashMap;
use std::fmt;

// 熔爐吃的咒語
#[derive(Clone, Copy, Debug, PartialEq)]
enum Spell {
    Go(&'static str),     // 往某個方向走一格
    Gather(&'static str), // 撿起一顆某種寶石
    Rest,                 // 休息符：只用來分段，不熔出任何指令
}

impl fmt::Display for Spell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Spell::Go(dir) => write!(f, "MOVE {}", dir),
            Spell::Gather(_) => write!(f, "PICK"),
            Spell::Rest => write!(f, "（休息）"),
        }
    }
}

// 秤一道咒語：寶石記一筆帳，再把咒語交回
fn weigh(tally: &mut HashMap<&'static str, u32>, spell: Spell) -> Spell {
    if let Spell::Gather(kind) = spell {
        *tally.entry(kind).or_insert(0) += 1;
    }
    spell
}

// 熔一道咒語：休息符跳過，其他熔成一行字記進 words
macro_rules! forge {
    ($words:ident, $e:expr) => {
        // 參數只求值一次
        let spell = $e;
        match spell {
            Spell::Rest => {}
            _ => $words.push(spell.to_string()),
        }
    };
}

// 熔爐：把整卷咒語熔成字句，寶石記到帳本 tally
fn smelt(scroll: &[Spell], tally: &mut HashMap<&'static str, u32>) -> Vec<String> {
    let mut words = Vec::new();
    for &spell in scroll {
        forge!(words, weigh(tally, spell));
    }
    words
}

// 保險起見：只把 Ferris 認得的指令交出去
fn commands(words: Vec<String>) -> Vec<String> {
    let dirs = ["UP", "DOWN", "LEFT", "RIGHT"];
    words
        .into_iter()
        .filter(|w| w == "PICK" || dirs.iter().any(|d| *w == format!("MOVE {}", d)))
        .collect()
}

// 工坊的大捲軸：三段路，用休息符隔開
fn great_scroll() -> Vec<Spell> {
    use Spell::*;
    vec![
        Go("RIGHT"), Go("RIGHT"),
        Rest,
        Go("DOWN"), Gather("珍珠"), Go("DOWN"), Go("RIGHT"), Go("RIGHT"), Gather("海玻璃"),
        Rest,
        Go("UP"), Go("UP"), Go("RIGHT"), Go("RIGHT"), Gather("珍珠"), Go("DOWN"), Go("DOWN"), Go("DOWN"),
    ]
}

fn route() -> Vec<String> {
    let mut tally = HashMap::new();
    commands(smelt(&great_scroll(), &mut tally))
}

fn main() {
    let mut tally = HashMap::new();
    let words = smelt(&great_scroll(), &mut tally);
    println!(
        "帳本：珍珠 {} 顆、海玻璃 {} 顆",
        tally.get("珍珠").unwrap_or(&0),
        tally.get("海玻璃").unwrap_or(&0)
    );
    for c in commands(words) {
        println!("{}", c);
    }
}
