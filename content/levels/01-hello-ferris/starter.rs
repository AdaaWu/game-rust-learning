// 回傳 Ferris 要執行的指令
fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.push(String::from("MOVE DOWN"));
    // TODO 繼續加指令，帶 Ferris 走到終點
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
