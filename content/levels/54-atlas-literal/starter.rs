use std::collections::HashMap;

// 還沒畫好的海圖與還沒刻好的比較咒語（佔位用，刻好後可以刪掉）
fn unwritten_atlas() -> HashMap<String, (i32, i32)> {
    todo!("用 atlas! 的每一組 key => value 建出 HashMap")
}

fn unwritten_max<T>() -> T {
    todo!("找出最大的那一個")
}

// 海圖：atlas!{ "珊瑚島" => (1, 0), ... } 回傳 HashMap<String, (i32, i32)>，可以有結尾逗號
// 比對規則與展開內容都由你刻：這條規則什麼都收，只是佔位
macro_rules! atlas {
    ($($unwritten:tt)*) => {
        unwritten_atlas()
    };
}

// 最大值：max_of!(a, b, c, ...) 至少一個，可以有結尾逗號，整數和小數都要能比
// 比對規則與展開內容都由你刻：這條規則什麼都收，只是佔位
macro_rules! max_of {
    ($($unwritten:tt)*) => {
        unwritten_max()
    };
}

// 從 from 航向 to：先左右、再上下
fn sail(from: (i32, i32), to: (i32, i32)) -> Vec<String> {
    let mut cmds = Vec::new();
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let side = if dx > 0 { "RIGHT" } else { "LEFT" };
    let updown = if dy > 0 { "DOWN" } else { "UP" };
    for _ in 0..dx.abs() {
        cmds.push(format!("MOVE {}", side));
    }
    for _ in 0..dy.abs() {
        cmds.push(format!("MOVE {}", updown));
    }
    cmds
}

// 島離出發點 (0, 0) 有幾格遠（只能上下左右走）
fn reach(p: (i32, i32)) -> i32 {
    p.0.abs() + p.1.abs()
}

fn route() -> Vec<String> {
    let atlas: HashMap<String, (i32, i32)> = atlas! {
        "珊瑚島" => (3, 0),
        "海草島" => (1, 3),
        "燈塔島" => (6, 2),
    };
    let farthest: i32 = max_of!(
        reach(atlas["珊瑚島"]),
        reach(atlas["海草島"]),
        reach(atlas["燈塔島"]),
    );
    // 由近到遠拜訪每座島：沿途的島各撿一顆寶石，最遠的島就是終點
    let mut isles: Vec<(i32, i32)> = atlas.values().copied().collect();
    isles.sort_by_key(|&p| reach(p));
    let mut cmds = Vec::new();
    let mut at = (0, 0);
    for isle in isles {
        cmds.extend(sail(at, isle));
        at = isle;
        if reach(isle) < farthest {
            cmds.push("PICK".to_string());
        }
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
