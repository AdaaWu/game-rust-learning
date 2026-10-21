// 會移動的夥伴
trait Mover {
    fn name(&self) -> String;

    // 往 dir 移動一次要印的指令
    fn step(&self, dir: &str) -> Vec<String>;

    // 往 dir 移動 n 次
    fn travel(&self, dir: &str, n: usize) -> Vec<String> {
        let mut cmds = Vec::new();
        for _ in 0..n {
            cmds.extend(self.step(dir));
        }
        cmds
    }
}

struct Ferris;
struct Seagull;

impl Mover for Ferris {
    fn name(&self) -> String {
        "Ferris".to_string()
    }

    // 螃蟹一次走一格
    fn step(&self, dir: &str) -> Vec<String> {
        vec![format!("MOVE {}", dir)]
    }
}

impl Mover for Seagull {
    fn name(&self) -> String {
        "海鷗".to_string()
    }

    // 海鷗拍一下翅膀飛兩格
    fn step(&self, dir: &str) -> Vec<String> {
        vec![format!("MOVE {}", dir), format!("MOVE {}", dir)]
    }
}

// 報名出發：借用任何一位夥伴，不管牠是哪個型別
fn announce(m: &dyn Mover) -> String {
    format!("{} 出發！", m.name())
}

// 接力：隊伍裡每位夥伴依序報名，再往 dir 移動 n 次
fn relay(team: &[Box<dyn Mover>], dir: &str, n: usize) -> Vec<String> {
    let mut cmds = Vec::new();
    for m in team {
        cmds.push(announce(m.as_ref()));
        cmds.extend(m.travel(dir, n));
    }
    cmds
}

fn route() -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.push(announce(&Ferris));
    cmds.extend(Ferris.travel("RIGHT", 1));
    cmds.push("PICK".to_string());
    let team: Vec<Box<dyn Mover>> = vec![Box::new(Ferris), Box::new(Seagull)];
    cmds.extend(relay(&team, "RIGHT", 1));
    cmds.push("PICK".to_string());
    let team: Vec<Box<dyn Mover>> = vec![Box::new(Seagull), Box::new(Ferris)];
    cmds.extend(relay(&team, "DOWN", 1));
    cmds.extend(Ferris.travel("RIGHT", 2));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
