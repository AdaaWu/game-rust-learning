// 沿路每一格的寶箱，由左到右：Some(寶石名) 或 None（空箱）
const CHESTS: [Option<&str>; 5] = [None, Some("珍珠"), None, Some("海玻璃"), None];

// 有寶石回傳「寶箱裡有<名字>」，空箱回傳「空箱子」
fn describe(chest: Option<&str>) -> String {
    todo!("用 match 處理 Some 與 None")
}

// 一路往右開寶箱，有寶石才 PICK，最後往下走到終點
fn route(chests: &[Option<&str>]) -> Vec<String> {
    let mut cmds = Vec::new();
    for chest in chests {
        cmds.push("MOVE RIGHT".to_string());
        todo!("用 if let 檢查寶箱，有寶石才 PICK")
    }
    cmds.push("MOVE DOWN".to_string());
    cmds
}

fn main() {
    for chest in CHESTS {
        println!("{}", describe(chest));
    }
    for c in route(&CHESTS) {
        println!("{}", c);
    }
}
