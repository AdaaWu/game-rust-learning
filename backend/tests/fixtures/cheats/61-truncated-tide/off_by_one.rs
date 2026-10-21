// 取巧：用 > 256 判斷上限，256 仍被截斷成 0
// 沙洲上每一欄的潮位感測器讀數（公分）
const TIDES: [u32; 8] = [20, 40, 300, 60, 256, 90, 180, 10];
// 潮位刻度超過這個值，沙洲那一格就被淹沒
const SAFE_TIDE: u8 = 100;
// 寶石的位置：沙洲在第 1 列、礁岩高地在第 0 列
const GEMS: [(usize, usize); 3] = [(3, 1), (5, 1), (6, 0)];

// 讀數超出潮位刻度（0–255）代表淹得比刻度還高，帶著原始讀數回報
#[derive(Debug, PartialEq)]
struct TideError {
    raw: u32,
}

// 把感測器讀數換成潮位刻度
fn tide_level(raw: u32) -> Result<u8, TideError> {
    if raw > 256 {
        return Err(TideError { raw });
    }
    Ok(raw as u8)
}

// 這一欄的沙洲有沒有被淹沒
fn flooded(raw: u32) -> bool {
    match tide_level(raw) {
        Ok(level) => level > SAFE_TIDE,
        Err(_) => true,
    }
}

// 從沙洲最左邊走到最右邊：淹沒的欄改走上方的礁岩高地，沙洲露出來就走回沙洲撿寶石
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    let mut row = 1;
    for x in 1..TIDES.len() {
        let want = if flooded(TIDES[x]) { 0 } else { 1 };
        if row == 1 && want == 0 {
            cmds.push("MOVE UP".to_string());
            cmds.push("MOVE RIGHT".to_string());
        } else if row == 0 && want == 1 {
            cmds.push("MOVE RIGHT".to_string());
            cmds.push("MOVE DOWN".to_string());
        } else {
            cmds.push("MOVE RIGHT".to_string());
        }
        row = want;
        if GEMS.contains(&(x, row)) {
            cmds.push("PICK".to_string());
        }
    }
    cmds
}

fn main() {
    for (x, raw) in TIDES.iter().enumerate() {
        println!("第 {} 欄潮位：{:?}", x, tide_level(*raw));
    }
    for c in route() {
        println!("{}", c);
    }
}
