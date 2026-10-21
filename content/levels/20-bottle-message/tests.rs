#[test]
fn from_parse_int_error() {
    let raw = "abc".parse::<u32>().unwrap_err();
    let e: BottleError = raw.clone().into();
    ::std::assert!(
        ::std::matches!(&e, BottleError::BadSteps(kept) if *kept == raw),
        "ParseIntError 應該轉成 BadSteps 並保留原本的錯誤，實際 {:?}",
        e
    );
}

#[test]
fn parse_valid_line() {
    let got = parse_line("RIGHT 3");
    ::std::assert!(::std::matches!(got, Ok((Dir::Right, 3))), "RIGHT 3 應該讀成往右 3 步，實際 {:?}", got);
    let got = parse_line("  UP   12 ");
    ::std::assert!(::std::matches!(got, Ok((Dir::Up, 12))), "多餘的空白不該影響解讀，實際 {:?}", got);
}

#[test]
fn empty_line_is_empty() {
    let got = parse_line("   ");
    ::std::assert!(::std::matches!(got, Err(BottleError::Empty)), "空白的信應該回報 Empty，實際 {:?}", got);
}

#[test]
fn unknown_direction_named() {
    let got = parse_line("NORTH 2");
    ::std::assert!(
        ::std::matches!(&got, Err(BottleError::UnknownDir(word)) if word == "NORTH"),
        "看不懂的方向要把原字帶回來，實際 {:?}",
        got
    );
}

#[test]
fn bad_or_missing_steps() {
    ::std::assert!(::std::matches!(parse_line("LEFT two"), Err(BottleError::BadSteps(_))), "步數不是數字應該回報 BadSteps");
    ::std::assert!(::std::matches!(parse_line("DOWN"), Err(BottleError::BadSteps(_))), "少了步數也應該回報 BadSteps");
}

#[test]
fn decode_whole_letter() {
    ::std::assert_eq!(
        decode("RIGHT 2\nDOWN 1"),
        Ok(::std::vec!["MOVE RIGHT".to_string(), "MOVE RIGHT".to_string(), "MOVE DOWN".to_string()]),
        "整封信應該展開成一步一步的指令"
    );
    let got = decode("UP 1\nNORTH 2");
    ::std::assert!(
        ::std::matches!(&got, Err(BottleError::UnknownDir(word)) if word == "NORTH"),
        "任何一行讀不懂，整封信都要退回，實際 {:?}",
        got
    );
}
