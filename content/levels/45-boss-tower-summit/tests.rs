type ParseFn = fn(&str) -> Result<Step, TowerError>;

const DIR_CHARS: [(char, Dir); 4] = [('U', Dir::Up), ('D', Dir::Down), ('L', Dir::Left), ('R', Dir::Right)];

#[derive(Clone, Copy, PartialEq)]
enum Flaw {
    Correct,
    BadAsZero,
    BadAsMissing,
    MissingAsZero,
    UnknownAsRight,
    OffByOne,
}

// 正確的讀法，以及故意改壞的幾種版本
fn flawed_parse(s: &str, flaw: Flaw) -> Result<Step, TowerError> {
    let s = s.trim();
    let mut chars = s.chars();
    let dir = match chars.next() {
        Some('U') => Dir::Up,
        Some('D') => Dir::Down,
        Some('L') => Dir::Left,
        Some('R') => Dir::Right,
        Some(_) if flaw == Flaw::UnknownAsRight => Dir::Right,
        Some(other) => return Err(TowerError::UnknownDir(other)),
        None => return Err(TowerError::Missing),
    };
    let digits = chars.as_str();
    if digits.is_empty() {
        if flaw == Flaw::MissingAsZero {
            return Ok(Step { dir, count: 0 });
        }
        return Err(TowerError::Missing);
    }
    let count = match digits.parse::<u32>() {
        Ok(n) if flaw == Flaw::OffByOne => n + 1,
        Ok(n) => n,
        Err(_) if flaw == Flaw::BadAsZero => 0,
        Err(_) if flaw == Flaw::BadAsMissing => return Err(TowerError::Missing),
        Err(e) => return Err(TowerError::BadNumber(e)),
    };
    Ok(Step { dir, count })
}

fn parse_reference(s: &str) -> Result<Step, TowerError> {
    flawed_parse(s, Flaw::Correct)
}

fn mutant_bad_as_zero(s: &str) -> Result<Step, TowerError> {
    flawed_parse(s, Flaw::BadAsZero)
}

fn mutant_bad_as_missing(s: &str) -> Result<Step, TowerError> {
    flawed_parse(s, Flaw::BadAsMissing)
}

fn mutant_missing_as_zero(s: &str) -> Result<Step, TowerError> {
    flawed_parse(s, Flaw::MissingAsZero)
}

fn mutant_unknown_as_right(s: &str) -> Result<Step, TowerError> {
    flawed_parse(s, Flaw::UnknownAsRight)
}

fn mutant_off_by_one(s: &str) -> Result<Step, TowerError> {
    flawed_parse(s, Flaw::OffByOne)
}

// 執行玩家的 check_parse，暫時關掉 panic 訊息
fn run_check(parse: ParseFn) -> Result<(), String> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(move || check_parse(parse));
    std::panic::set_hook(previous);
    outcome.map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

// 先修好 parse_step 的兩個 bug，再檢查 check_parse
fn require_parser_fixed() {
    ::std::assert!(
        ::std::matches!(parser::parse_step("Rx"), Err(TowerError::BadNumber(_))),
        "先修好 parse_step：Rx 應該回報 BadNumber，修好後這個測試才會檢查你的 check_parse"
    );
}

fn assert_caught(name: &str, mutant: ParseFn) {
    require_parser_fixed();
    ::std::assert!(run_check(mutant).is_err(), "你的 check_parse 沒抓到：{}", name);
}

#[test]
fn route_reaches_summit() {
    assert_cleared(&level_grid(), &planner::route());
}

#[test]
fn parse_reads_every_step() {
    for (ch, dir) in DIR_CHARS {
        for count in 0..=40u32 {
            for text in [::std::format!("{}{}", ch, count), ::std::format!("  {}{} ", ch, count)] {
                let got = parser::parse_step(&text);
                let same_dir = |d: &Dir| std::mem::discriminant(d) == std::mem::discriminant(&dir);
                ::std::assert!(
                    ::std::matches!(&got, Ok(Step { dir: d, count: c }) if same_dir(d) && *c == count),
                    "{:?} 應該讀成 {:?} 走 {} 格，實際 {:?}",
                    text,
                    dir,
                    count,
                    got
                );
            }
        }
    }
}

#[test]
fn bad_number_is_reported() {
    for text in ["Rx", "U3a", "L-1", "D 2", "R三", "U99999999999", "L2.5"] {
        match parser::parse_step(text) {
            Ok(step) => ::std::panic!("壞數字被吞掉了：{:?} 被讀成 {:?}", text, step),
            Err(TowerError::BadNumber(e)) => {
                ::std::assert_eq!(e, text[1..].parse::<u32>().unwrap_err(), "BadNumber 裡要放 parse 產生的那個錯誤");
            }
            Err(other) => ::std::panic!("錯誤種類應為 BadNumber：{:?} 卻回報 {:?}", text, other),
        }
    }
}

#[test]
fn missing_and_unknown_are_reported() {
    for text in ["", "   ", "R", " U "] {
        let got = parser::parse_step(text);
        ::std::assert!(::std::matches!(got, Err(TowerError::Missing)), "{:?} 少了方向或步數，應該是 Missing，實際 {:?}", text, got);
    }
    for (text, ch) in [("X3", 'X'), ("r2", 'r'), ("北4", '北')] {
        let got = parser::parse_step(text);
        ::std::assert!(
            ::std::matches!(got, Err(TowerError::UnknownDir(c)) if c == ch),
            "{:?} 的方向看不懂，實際 {:?}",
            text,
            got
        );
    }
}

#[test]
fn bad_number_keeps_source() {
    let err = parser::parse_step("Rx").expect_err("Rx 應該失敗");
    let source = std::error::Error::source(&err).expect("BadNumber 的 source() 應該是 Some");
    ::std::assert!(
        source.downcast_ref::<std::num::ParseIntError>().is_some(),
        "BadNumber 的源頭應該是 ParseIntError"
    );
}

#[test]
fn build_is_transactional() {
    let before = ::std::vec!["PICK".to_string(), "MOVE DOWN".to_string()];
    let good = ["R2", "U1", "L3"];
    let mut cmds = before.clone();
    ::std::assert_eq!(planner::build(&good, &mut cmds), Ok(()), "完好的計畫應該回傳 Ok");
    let mut expected = before.clone();
    for word in ["MOVE RIGHT", "MOVE RIGHT", "MOVE UP", "MOVE LEFT", "MOVE LEFT", "MOVE LEFT"] {
        expected.push(word.to_string());
    }
    ::std::assert_eq!(cmds, expected, "計畫的指令要依序接在原本的指令後面");
    for bad_at in 0..=good.len() {
        let mut plan: Vec<&str> = good.to_vec();
        plan.insert(bad_at, "Dx");
        let mut cmds = before.clone();
        ::std::assert!(planner::build(&plan, &mut cmds).is_err(), "計畫 {:?} 有壞步驟，應該回傳 Err", plan);
        ::std::assert_eq!(cmds, before, "計畫 {:?} 失敗時，cmds 應該和呼叫前一模一樣", plan);
    }
}

#[test]
fn check_parse_accepts_correct_parser() {
    if let Err(msg) = run_check(parse_reference) {
        ::std::panic!("你的 check_parse 把正確的 parse_step 也判成錯：{}", msg);
    }
}

#[test]
fn check_parse_catches_bad_number_as_zero() {
    assert_caught("把壞數字當 0", mutant_bad_as_zero);
}

#[test]
fn check_parse_catches_bad_number_as_missing() {
    assert_caught("壞數字回報成 Missing", mutant_bad_as_missing);
}

#[test]
fn check_parse_catches_missing_as_zero() {
    assert_caught("沒寫步數卻當成走 0 格", mutant_missing_as_zero);
}

#[test]
fn check_parse_catches_unknown_as_right() {
    assert_caught("看不懂的方向當成往右", mutant_unknown_as_right);
}

#[test]
fn check_parse_catches_off_by_one() {
    assert_caught("步數多算一格", mutant_off_by_one);
}
