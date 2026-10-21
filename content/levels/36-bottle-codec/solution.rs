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

// 速記 → Step：有了 FromStr，就能寫 tok.parse::<Step>()
impl FromStr for Step {
    type Err = StepError;

    fn from_str(s: &str) -> Result<Step, StepError> {
        match s {
            "R" => Ok(Step::Right),
            "L" => Ok(Step::Left),
            "U" => Ok(Step::Up),
            "D" => Ok(Step::Down),
            "*" => Ok(Step::Pick),
            other => Err(StepError::BadToken(other.to_string())),
        }
    }
}

// Step → Ferris 看得懂的指令：有了 Display，就能寫 format!("{step}") 與 step.to_string()
impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Step::Right => "MOVE RIGHT",
            Step::Left => "MOVE LEFT",
            Step::Up => "MOVE UP",
            Step::Down => "MOVE DOWN",
            Step::Pick => "PICK",
        };
        write!(f, "{}", text)
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
