fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.push(String::from("MOVE DOWN"));
    cmds.push(String::from("MOVE DOWN"));
    for _ in 0..4 {
        cmds.push(String::from("MOVE RIGHT"));
    }
    cmds.push(String::from("MOVE UP"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
