fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    for c in ["MOVE DOWN", "MOVE DOWN", "PICK", "MOVE RIGHT", "MOVE RIGHT", "MOVE RIGHT", "OPEN UP", "MOVE UP", "MOVE UP"] {
        cmds.push(String::from(c));
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
