// Ferris 的起點欄位
const START_X: usize = 7;
// 海流圖：每一段往左游幾格
const CHART: [usize; 4] = [2, 3, 4, 1];

// 從 x 往左游 n 格後的落點，游出岸邊（小於 0）就回傳 None
fn back_off(x: usize, n: usize) -> Option<usize> {
    x.checked_sub(n)
}

// 照海流圖往左游，每段終點撿珍珠；游不到的那段跳過，最後往下進港
fn route(start_x: usize, chart: &[usize]) -> Vec<String> {
    let mut x = start_x;
    let mut cmds = Vec::new();
    for &n in chart {
        match back_off(x, n) {
            Some(next) => {
                for _ in 0..n {
                    cmds.push("MOVE LEFT".to_string());
                }
                cmds.push("PICK".to_string());
                x = next;
            }
            None => cmds.push(format!("逆流 {} 格太強，抓緊岩石", n)),
        }
    }
    cmds.push("MOVE DOWN".to_string());
    cmds
}

fn main() {
    for c in route(START_X, &CHART) {
        println!("{}", c);
    }
}
