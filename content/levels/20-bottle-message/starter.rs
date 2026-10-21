use std::num::ParseIntError;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// 瓶中信讀不懂的原因
#[derive(Debug, PartialEq)]
enum BottleError {
    Empty,
    UnknownDir(String),
    BadSteps(ParseIntError),
}

// 讓 ? 能把 ParseIntError 自動轉成 BottleError
impl From<ParseIntError> for BottleError {
    fn from(e: ParseIntError) -> Self {
        todo!("包成 BottleError::BadSteps")
    }
}

fn parse_dir(word: &str) -> Result<Dir, BottleError> {
    match word {
        "UP" => Ok(Dir::Up),
        "DOWN" => Ok(Dir::Down),
        "LEFT" => Ok(Dir::Left),
        "RIGHT" => Ok(Dir::Right),
        other => Err(BottleError::UnknownDir(other.to_string())),
    }
}

// 解析一行密語，例如 "RIGHT 3" 回傳 (Dir::Right, 3)
fn parse_line(line: &str) -> Result<(Dir, u32), BottleError> {
    todo!("拆出方向與步數，用 ? 傳遞錯誤")
}

fn command(d: Dir) -> String {
    let text = match d {
        Dir::Up => "MOVE UP",
        Dir::Down => "MOVE DOWN",
        Dir::Left => "MOVE LEFT",
        Dir::Right => "MOVE RIGHT",
    };
    text.to_string()
}

// 把整封信翻成 Ferris 指令，任何一行讀不懂就整封退回
fn decode(letter: &str) -> Result<Vec<String>, BottleError> {
    let mut cmds = Vec::new();
    for line in letter.lines() {
        let (dir, steps) = parse_line(line)?;
        for _ in 0..steps {
            cmds.push(command(dir));
        }
    }
    Ok(cmds)
}

fn main() {
    let letters = ["RIGHT 2\nDOWN 1", "UP 1\nNORTH 2", "LEFT two"];
    for letter in letters {
        match decode(letter) {
            Ok(cmds) => println!("解讀成功：{:?}", cmds),
            Err(e) => println!("這封信讀不懂：{:?}", e),
        }
    }
}
