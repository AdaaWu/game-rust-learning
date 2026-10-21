mod parser {
    use std::error::Error;
    use std::fmt;
    use std::num::ParseIntError;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Dir {
        Up,
        Down,
        Left,
        Right,
    }

    // 一步：往哪個方向走幾格，例如 "R3"
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Step {
        pub dir: Dir,
        pub count: u32,
    }

    #[derive(Debug, PartialEq)]
    pub enum TowerError {
        Missing,
        BadNumber(ParseIntError),
        UnknownDir(char),
    }

    impl fmt::Display for TowerError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                TowerError::Missing => write!(f, "少了方向或步數"),
                TowerError::BadNumber(_) => write!(f, "步數不是數字"),
                TowerError::UnknownDir(c) => write!(f, "看不懂方向 {}", c),
            }
        }
    }

    impl Error for TowerError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                TowerError::BadNumber(e) => Some(e),
                _ => None,
            }
        }
    }

    // 讓 ? 把 ParseIntError 轉成 TowerError
    impl From<ParseIntError> for TowerError {
        fn from(e: ParseIntError) -> Self {
            TowerError::BadNumber(e)
        }
    }

    // 讀一步，例如 "R3" 是往右 3 格
    pub fn parse_step(s: &str) -> Result<Step, TowerError> {
        let s = s.trim();
        let mut chars = s.chars();
        let dir = match chars.next() {
            Some('U') => Dir::Up,
            Some('D') => Dir::Down,
            Some('L') => Dir::Left,
            Some('R') => Dir::Right,
            Some(other) => return Err(TowerError::UnknownDir(other)),
            None => return Err(TowerError::Missing),
        };
        let digits = chars.as_str();
        if digits.is_empty() {
            return Err(TowerError::Missing);
        }
        let count = digits.parse::<u32>()?;
        Ok(Step { dir, count })
    }
}

mod planner {
    use crate::parser::{self, Dir, TowerError};

    // 每層樓：主計畫、備援計畫
    const FLOORS: [(&[&str], &[&str]); 3] = [
        (&["Rx", "U2"], &["R3", "U2"]),
        (&["L2", "U2"], &["L2", "U2"]),
        (&["R4", "U?"], &["R4", "U2"]),
    ];

    fn word(dir: Dir) -> &'static str {
        match dir {
            Dir::Up => "MOVE UP",
            Dir::Down => "MOVE DOWN",
            Dir::Left => "MOVE LEFT",
            Dir::Right => "MOVE RIGHT",
        }
    }

    // 整份計畫都讀懂了才寫進 cmds
    pub fn build(plan: &[&str], cmds: &mut Vec<String>) -> Result<(), TowerError> {
        let steps = plan
            .iter()
            .map(|s| parser::parse_step(s))
            .collect::<Result<Vec<_>, _>>()?;
        for step in steps {
            for _ in 0..step.count {
                cmds.push(word(step.dir).to_string());
            }
        }
        Ok(())
    }

    // 主計畫讀不懂就改用備援計畫；每層樓梯口撿一顆寶石
    pub fn route() -> Vec<String> {
        let mut cmds = Vec::new();
        for (i, (main, backup)) in FLOORS.iter().enumerate() {
            if build(main, &mut cmds).is_err() {
                build(backup, &mut cmds).expect("備援計畫不該讀不懂");
            }
            if i + 1 < FLOORS.len() {
                cmds.push("PICK".to_string());
            }
        }
        cmds
    }
}

use parser::{Dir, Step, TowerError};

// 我之前寫的檢查：parse_step 寫錯就 panic
fn check_parse(parse: fn(&str) -> Result<Step, TowerError>) {
    assert_eq!(parse("R3"), Ok(Step { dir: Dir::Right, count: 3 }), "R3 是往右 3 格");
    assert_eq!(parse("U12"), Ok(Step { dir: Dir::Up, count: 12 }), "U12 是往上 12 格");
    assert_eq!(parse(" L1 "), Ok(Step { dir: Dir::Left, count: 1 }), "前後空白要忽略");
    assert!(matches!(parse("Rx"), Err(TowerError::BadNumber(_))), "壞數字要回報 BadNumber");
    assert!(matches!(parse("D3a"), Err(TowerError::BadNumber(_))), "數字後面有雜字也是 BadNumber");
    assert_eq!(parse("R"), Err(TowerError::Missing), "只有方向沒有步數是 Missing");
    assert_eq!(parse(""), Err(TowerError::Missing), "空字串是 Missing");
    assert_eq!(parse("X3"), Err(TowerError::UnknownDir('X')), "看不懂的方向要回報");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_step_is_correct() {
        check_parse(parser::parse_step);
    }
}

fn main() {
    check_parse(parser::parse_step);
    println!("check_parse 通過");
    for c in planner::route() {
        println!("{}", c);
    }
}
