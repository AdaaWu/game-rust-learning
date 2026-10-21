// 測試用的固定亂數，產生多組計畫
fn next_seed(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    *seed >> 16
}

fn parse_error_of(s: &str) -> ParseIntError {
    s.trim().parse::<u32>().unwrap_err()
}

// 沿著 source() 走完整條錯誤鏈
fn chain_of(err: &dyn Error) -> Vec<String> {
    let mut texts = ::std::vec![err.to_string()];
    let mut cause = err.source();
    while let Some(c) = cause {
        texts.push(c.to_string());
        cause = c.source();
    }
    texts
}

#[test]
fn read_floor_reads_numbers() {
    for n in 0..=30u32 {
        for text in [::std::format!("{}", n), ::std::format!(" {}", n), ::std::format!("{} ", n), ::std::format!("  {}  ", n)] {
            ::std::assert!(
                ::std::matches!(read_floor(&text), Ok(v) if v == n),
                "read_floor({:?}) 應該回傳 Ok({})",
                text,
                n
            );
        }
    }
}

#[test]
fn read_floor_reports_missing() {
    for text in ["", " ", "   "] {
        ::std::assert!(
            ::std::matches!(read_floor(text), Err(FloorError::Missing)),
            "read_floor({:?}) 是空的，應該回傳 Missing，實際 {:?}",
            text,
            read_floor(text)
        );
    }
}

#[test]
fn read_floor_wraps_parse_error() {
    for text in ["三", "x2", "-1", "3.5", " 2a ", "九十"] {
        match read_floor(text) {
            Err(FloorError::BadNumber(e)) => {
                ::std::assert_eq!(e, parse_error_of(text), "BadNumber 裡要放 parse 產生的那個錯誤");
            }
            other => ::std::panic!("read_floor({:?}) 應該回傳 BadNumber，實際 {:?}", text, other),
        }
    }
}

#[test]
fn floor_error_source_points_to_parse_error() {
    let err = read_floor("三").expect_err("「三」不是數字");
    let source = err.source().expect("BadNumber 的 source() 應該是 Some");
    ::std::assert!(
        source.downcast_ref::<ParseIntError>().is_some(),
        "BadNumber 的源頭應該是 ParseIntError"
    );
    ::std::assert!(FloorError::Missing.source().is_none(), "Missing 沒有源頭，source() 應該是 None");
}

#[test]
fn climb_follows_generated_plans() {
    let mut seed = 41;
    for _ in 0..60 {
        let parts = next_seed(&mut seed) % 5 + 1;
        let floors: Vec<u32> = (0..parts).map(|_| next_seed(&mut seed) % 6).collect();
        let plan = floors.iter().map(|n| ::std::format!(" {} ", n)).collect::<Vec<_>>().join(",");
        let total: u32 = floors.iter().sum();
        let cmds = climb(&plan).unwrap_or_else(|e| ::std::panic!("計畫 {:?} 應該爬得上去，卻回傳 {:?}", plan, e));
        ::std::assert_eq!(cmds.len() as u32, total, "計畫 {:?} 一共要爬 {} 層", plan, total);
        ::std::assert!(cmds.iter().all(|c| c == "MOVE UP"), "每層一個 MOVE UP");
    }
}

#[test]
fn climb_reports_too_high() {
    for (plan, high) in [("6", 6), ("1,9,2", 9), ("2, 42 ,x", 42), ("5,5,7,8", 7)] {
        ::std::assert!(
            ::std::matches!(climb(plan), Err(ClimbError::TooHigh(n)) if n == high),
            "計畫 {:?} 應該在 {} 層時回報 TooHigh，實際 {:?}",
            plan,
            high,
            climb(plan)
        );
    }
}

#[test]
fn climb_wraps_floor_error() {
    ::std::assert!(
        ::std::matches!(climb("1,,2"), Err(ClimbError::Floor(FloorError::Missing))),
        "「1,,2」中間少一段，應該是 Floor(Missing)，實際 {:?}",
        climb("1,,2")
    );
    ::std::assert!(
        ::std::matches!(climb("2,三"), Err(ClimbError::Floor(FloorError::BadNumber(_)))),
        "「2,三」應該是 Floor(BadNumber)，實際 {:?}",
        climb("2,三")
    );
    let err = climb("2,三").unwrap_err();
    let source = err.source().expect("Floor 的 source() 應該是 Some");
    ::std::assert!(
        source.downcast_ref::<FloorError>().is_some(),
        "ClimbError::Floor 的源頭應該是 FloorError"
    );
    ::std::assert!(ClimbError::TooHigh(9).source().is_none(), "TooHigh 沒有源頭，source() 應該是 None");
}

#[test]
fn display_tells_what_happened() {
    for n in [6, 9, 42, 1234] {
        let text = ClimbError::TooHigh(n).to_string();
        ::std::assert!(text.contains(&n.to_string()), "TooHigh({}) 的訊息應該說出層數，實際 {:?}", n, text);
    }
    let missing = FloorError::Missing.to_string();
    let bad = FloorError::BadNumber(parse_error_of("三")).to_string();
    ::std::assert!(!missing.trim().is_empty() && !bad.trim().is_empty(), "每種錯誤都要有說明文字");
    ::std::assert_ne!(missing, bad, "Missing 和 BadNumber 的說明應該不一樣");
    ::std::assert!(!ClimbError::Floor(FloorError::Missing).to_string().trim().is_empty(), "Floor 也要有說明文字");
}

#[test]
fn run_keeps_error_type_in_box() {
    let err = run("2,三").expect_err("「2,三」應該失敗");
    ::std::assert!(
        ::std::matches!(err.downcast_ref::<ClimbError>(), Some(ClimbError::Floor(FloorError::BadNumber(_)))),
        "Box<dyn Error> 裡應該還是 ClimbError::Floor(BadNumber)，不能先轉成字串"
    );
    let chain = chain_of(err.as_ref());
    ::std::assert_eq!(chain.len(), 3, "錯誤鏈應該有三層（ClimbError → FloorError → ParseIntError），實際 {:?}", chain);
    ::std::assert_eq!(chain[2], parse_error_of("三").to_string(), "錯誤鏈最底層應該是 ParseIntError");
    let err = run("3,8").expect_err("一次 8 層太高");
    ::std::assert!(
        ::std::matches!(err.downcast_ref::<ClimbError>(), Some(ClimbError::TooHigh(8))),
        "「3,8」應該是 TooHigh(8)"
    );
}

#[test]
fn run_rejects_empty_plan() {
    for plan in ["", "   "] {
        let err = run(plan).expect_err("空計畫應該失敗");
        ::std::assert_eq!(err.to_string(), "計畫是空的", "空計畫的訊息不對");
        ::std::assert!(err.downcast_ref::<ClimbError>().is_none(), "空計畫是 run 自己擋下的，不是 ClimbError");
    }
}

#[test]
fn run_climbs_good_plan() {
    let cmds = run("2,3").expect("「2,3」應該爬得上去");
    ::std::assert_eq!(cmds, ::std::vec!["MOVE UP"; 5], "「2,3」一共爬 5 層");
}
