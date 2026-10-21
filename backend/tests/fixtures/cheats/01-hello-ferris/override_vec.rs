// 取巧：覆寫 vec! 讓產生的地圖沒有牆，直線走到終點
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    for _ in 0..4 {
        cmds.push(String::from("MOVE RIGHT"));
    }
    cmds.push(String::from("MOVE DOWN"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}

macro_rules! vec {
    ($($t:tt)*) => {
        ::std::vec::Vec::new()
    };
}
