use std::fmt;
use std::str::FromStr;

// 瓶中信的速記：R L U D 是方向，* 是撿寶石
#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Right,
    Left,
    Up,
    Down,
    Pick,
}

#[derive(Debug, PartialEq)]
enum StepError {
    BadToken(String),
}

impl Step {
    // 速記 → Step
    fn from_text(s: &str) -> Result<Step, StepError> {
        match s {
            "R" => Ok(Step::Right),
            "L" => Ok(Step::Left),
            "U" => Ok(Step::Up),
            "D" => Ok(Step::Down),
            "*" => Ok(Step::Pick),
            other => Err(StepError::BadToken(other.to_string())),
        }
    }

    // Step → Ferris 看得懂的指令
    fn render(&self) -> String {
        match self {
            Step::Right => "MOVE RIGHT".to_string(),
            Step::Left => "MOVE LEFT".to_string(),
            Step::Up => "MOVE UP".to_string(),
            Step::Down => "MOVE DOWN".to_string(),
            Step::Pick => "PICK".to_string(),
        }
    }
}

// 解讀瓶中信：記號之間用空白隔開
fn route() -> Vec<String> {
    let letter = "R R * D D R R * U R R * D D D";
    let mut cmds = Vec::new();
    for tok in letter.split_whitespace() {
        let step = tok.parse::<Step>().unwrap();
        cmds.push(format!("{step}"));
    }
    cmds
}

fn main() {
    for c in route() {
        println!("{}", c);
    }
}
