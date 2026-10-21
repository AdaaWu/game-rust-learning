fn route() -> Vec<String> {
    let cmds = Vec::new();
    cmds.push(String::from("MOVE DOWN"));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
