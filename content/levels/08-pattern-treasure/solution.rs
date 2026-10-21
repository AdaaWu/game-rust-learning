#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Sand,
    Door,
    Gem(u32),
}

// Ferris 右邊依序的格子，最後一格是出口
const SHORE: [Tile; 6] = [
    Tile::Sand,
    Tile::Gem(3),
    Tile::Door,
    Tile::Gem(12),
    Tile::Gem(5),
    Tile::Sand,
];

// 鑑定一格的分數：10 分以上的稀有寶石加倍，沙地和門是 0 分
fn score(tile: Tile) -> u32 {
    match tile {
        Tile::Gem(v) if v >= 10 => v * 2,
        Tile::Gem(v) => v,
        Tile::Sand | Tile::Door => 0,
    }
}

// 從左邊走進這一格要下的指令：門要先打開，寶石要撿起來
fn enter(tile: Tile) -> Vec<String> {
    let words = match tile {
        Tile::Sand => vec!["MOVE RIGHT"],
        Tile::Door => vec!["OPEN RIGHT", "MOVE RIGHT"],
        Tile::Gem(_) => vec!["MOVE RIGHT", "PICK"],
    };
    words.iter().map(|w| w.to_string()).collect()
}

// 沿著長廊往右走，回傳 (指令, 總分)
fn appraise(shore: &[Tile]) -> (Vec<String>, u32) {
    let mut cmds = Vec::new();
    let mut total = 0;
    for tile in shore {
        cmds.extend(enter(*tile));
        total += score(*tile);
    }
    (cmds, total)
}

fn route() -> Vec<String> {
    let (cmds, _) = appraise(&SHORE);
    cmds
}

fn main() {
    let (cmds, total) = appraise(&SHORE);
    for c in cmds {
        println!("{}", c);
    }
    println!("鑑定總分：{}", total);
}
