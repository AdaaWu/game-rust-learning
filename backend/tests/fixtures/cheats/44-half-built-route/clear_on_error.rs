// 取巧：失敗時把整個 cmds 清空，連呼叫前的指令也一起丟掉
// 樓梯哪一階裂開了
#[derive(Debug, PartialEq)]
enum TowerError {
    Cracked { index: usize, ch: char },
}

// 樓梯圖上的一個字元對應一個指令
fn command(ch: char) -> Option<&'static str> {
    match ch {
        'R' => Some("MOVE RIGHT"),
        'L' => Some("MOVE LEFT"),
        'U' => Some("MOVE UP"),
        'D' => Some("MOVE DOWN"),
        'P' => Some("PICK"),
        _ => None,
    }
}

// 把一層樓的樓梯圖轉成指令，接到 cmds 後面；有裂開的階梯就回傳錯誤
fn append_floor(floor: &str, cmds: &mut Vec<String>) -> Result<(), TowerError> {
    for (index, ch) in floor.chars().enumerate() {
        match command(ch) {
            Some(cmd) => cmds.push(cmd.to_string()),
            None => {
                cmds.clear();
                return Err(TowerError::Cracked { index, ch });
            }
        }
    }
    Ok(())
}

// 每層樓有主樓梯和備援樓梯
const FLOORS: [(&str, &str); 3] = [
    ("RRPRUU", "RRPRUU"),
    ("LL*UU", "LLUU"),
    ("?RRRPUU", "RRRPUU"),
];

// 主樓梯有裂縫就改走備援樓梯
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    for (main, backup) in FLOORS {
        if append_floor(main, &mut cmds).is_err() {
            append_floor(backup, &mut cmds).expect("備援樓梯不該有裂縫");
        }
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
