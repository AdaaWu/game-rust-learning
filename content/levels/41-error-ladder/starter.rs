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
        todo!("BadNumber 與 Missing 各寫一句說明")
    }
}

impl Error for FloorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!("BadNumber 的源頭是裡面的 ParseIntError；Missing 沒有源頭")
    }
}

impl fmt::Display for ClimbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("Floor 與 TooHigh 各寫一句說明，TooHigh 要說出是幾層")
    }
}

impl Error for ClimbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!("Floor 的源頭是裡面的 FloorError；TooHigh 沒有源頭")
    }
}

// 讓 ? 把 ParseIntError 往上轉成 FloorError
impl From<ParseIntError> for FloorError {
    fn from(e: ParseIntError) -> Self {
        todo!("包成 FloorError::BadNumber")
    }
}

// 讓 ? 把 FloorError 往上轉成 ClimbError
impl From<FloorError> for ClimbError {
    fn from(e: FloorError) -> Self {
        todo!("包成 ClimbError::Floor")
    }
}

// 一次最多爬幾層
const MAX_STEP: u32 = 5;

// 讀一段樓層數，例如 " 3 " 回傳 3
fn read_floor(s: &str) -> Result<u32, FloorError> {
    // 去掉空白後是空的：回傳 Missing
    // 數字讀不懂：parse 的錯誤用 ? 自動轉成 BadNumber
    todo!("讀出樓層數")
}

// 依計畫爬塔，例如 "2,3" 是先爬 2 層再爬 3 層，每層一個 MOVE UP
fn climb(plan: &str) -> Result<Vec<String>, ClimbError> {
    // 用逗號切開，每段 read_floor(..)? 往上轉成 ClimbError
    // 一段超過 MAX_STEP：回傳 TooHigh(那一段的層數)
    todo!("依序爬每一段")
}

// 應用層：不管哪一種錯誤都裝進 Box<dyn Error>
fn run(plan: &str) -> Result<Vec<String>, Box<dyn Error>> {
    // 計畫去掉空白後是空的：回傳錯誤訊息 "計畫是空的"
    // 其餘交給 climb(plan)?
    todo!("爬塔")
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
