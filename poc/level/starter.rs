// 往右走四步、撿起寶石，再走一步到終點旗子
fn route() -> Vec<String> {
    let cmds = Vec::new();
    let steps = 0;
    while steps < 4 {
        cmds.push(String::from("MOVE RIGHT"));
        steps += 1;
    }
    cmds.push(String::from("PICK"));
    cmds.push(String::from("MOVE RIGHT"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
