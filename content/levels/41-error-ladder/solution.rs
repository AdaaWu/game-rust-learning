use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// 讀一段樓層數時可能出的錯
#[derive(Debug)]
enum FloorError {
    BadNumber(ParseIntError),
    Missing,
}

// 爬整座塔時可能出的錯
#[derive(Debug)]
enum ClimbError {
    Floor(FloorError),
    TooHigh(u32),
}

impl fmt::Display for FloorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FloorError::BadNumber(_) => write!(f, "樓層數不是數字"),
            FloorError::Missing => write!(f, "少寫了一段樓層數"),
        }
    }
}

impl Error for FloorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FloorError::BadNumber(e) => Some(e),
            FloorError::Missing => None,
        }
    }
}

impl fmt::Display for ClimbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClimbError::Floor(_) => write!(f, "有一段樓梯讀不懂"),
            ClimbError::TooHigh(n) => write!(f, "一次爬 {} 層太高了，最多 {} 層", n, MAX_STEP),
        }
    }
}

impl Error for ClimbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClimbError::Floor(e) => Some(e),
            ClimbError::TooHigh(_) => None,
        }
    }
}

// 讓 ? 把 ParseIntError 往上轉成 FloorError
impl From<ParseIntError> for FloorError {
    fn from(e: ParseIntError) -> Self {
        FloorError::BadNumber(e)
    }
}

// 讓 ? 把 FloorError 往上轉成 ClimbError
impl From<FloorError> for ClimbError {
    fn from(e: FloorError) -> Self {
        ClimbError::Floor(e)
    }
}

// 一次最多爬幾層
const MAX_STEP: u32 = 5;

// 讀一段樓層數，例如 " 3 " 回傳 3
fn read_floor(s: &str) -> Result<u32, FloorError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(FloorError::Missing);
    }
    Ok(s.parse::<u32>()?)
}

// 依計畫爬塔，例如 "2,3" 是先爬 2 層再爬 3 層，每層一個 MOVE UP
fn climb(plan: &str) -> Result<Vec<String>, ClimbError> {
    let mut cmds = Vec::new();
    for part in plan.split(',') {
        let n = read_floor(part)?;
        if n > MAX_STEP {
            return Err(ClimbError::TooHigh(n));
        }
        for _ in 0..n {
            cmds.push("MOVE UP".to_string());
        }
    }
    Ok(cmds)
}

// 應用層：不管哪一種錯誤都裝進 Box<dyn Error>
fn run(plan: &str) -> Result<Vec<String>, Box<dyn Error>> {
    if plan.trim().is_empty() {
        return Err("計畫是空的".into());
    }
    Ok(climb(plan)?)
}

fn main() {
    for plan in ["2,3", "2,三", "1,,2", "4,7", ""] {
        match run(plan) {
            Ok(cmds) => println!("{:?}：爬了 {} 層", plan, cmds.len()),
            Err(e) => {
                println!("{:?}：{}", plan, e);
                let mut cause = e.source();
                while let Some(c) = cause {
                    println!("  原因：{}", c);
                    cause = c.source();
                }
            }
        }
    }
}
