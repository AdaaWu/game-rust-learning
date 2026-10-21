// 取巧：steps_with 不呼叫 f，直接回傳原本的步伐；lead 改成自己反轉方向
// 會走路的夥伴
trait Walker {
    fn name(&self) -> String;

    // 輪到自己時要走的指令
    fn steps(&self) -> Vec<String>;

    // 把每一步交給 f 改寫後回傳，例如把方向反過來
    // 泛型方法進不了 vtable：限定只有已知大小的具體型別能呼叫
    fn steps_with<F: Fn(&str) -> String>(&self, f: F) -> Vec<String>
    where
        Self: Sized,
    {
        let _ = f;
        self.steps()
    }
}

// Ferris 一次走一格
struct Ferris {
    dir: &'static str,
}

// 海鷗一次飛兩格
struct Seagull {
    dir: &'static str,
}

// 寄居蟹走一格就撿東西
struct Hermit {
    dir: &'static str,
}

impl Walker for Ferris {
    fn name(&self) -> String {
        "Ferris".to_string()
    }

    fn steps(&self) -> Vec<String> {
        vec![format!("MOVE {}", self.dir)]
    }
}

impl Walker for Seagull {
    fn name(&self) -> String {
        "海鷗".to_string()
    }

    fn steps(&self) -> Vec<String> {
        vec![format!("MOVE {}", self.dir), format!("MOVE {}", self.dir)]
    }
}

impl Walker for Hermit {
    fn name(&self) -> String {
        "寄居蟹".to_string()
    }

    fn steps(&self) -> Vec<String> {
        vec![format!("MOVE {}", self.dir), "PICK".to_string()]
    }
}

// 把一步的方向反過來
fn flip(step: &str) -> String {
    match step {
        "MOVE UP" => "MOVE DOWN".to_string(),
        "MOVE DOWN" => "MOVE UP".to_string(),
        "MOVE LEFT" => "MOVE RIGHT".to_string(),
        "MOVE RIGHT" => "MOVE LEFT".to_string(),
        other => other.to_string(),
    }
}

// 隊長探路（熱路徑，泛型靜態分派）：走過去撿寶石，再原路退回
fn lead<W: Walker>(w: &W) -> Vec<String> {
    let mut cmds = w.steps();
    cmds.push("PICK".to_string());
    let mut back: Vec<String> = w.steps().iter().map(|s| flip(s)).collect();
    back.reverse();
    cmds.extend(back);
    cmds
}

// 接力：隊伍裡每位夥伴先喊接棒，再走自己那一段
fn relay(squad: &[Box<dyn Walker>]) -> Vec<String> {
    let mut cmds = Vec::new();
    for w in squad {
        cmds.push(format!("{} 接棒", w.name()));
        cmds.extend(w.steps());
    }
    cmds
}

fn route() -> Vec<String> {
    let mut cmds = lead(&Ferris { dir: "DOWN" });
    let squad: Vec<Box<dyn Walker>> = vec![
        Box::new(Ferris { dir: "RIGHT" }),
        Box::new(Seagull { dir: "RIGHT" }),
        Box::new(Hermit { dir: "DOWN" }),
        Box::new(Seagull { dir: "RIGHT" }),
        Box::new(Hermit { dir: "DOWN" }),
        Box::new(Ferris { dir: "RIGHT" }),
    ];
    cmds.extend(relay(&squad));
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
