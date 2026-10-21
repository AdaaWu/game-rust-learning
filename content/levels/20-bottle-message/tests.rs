#[test]
fn from_parse_int_error() {
    let raw = "abc".parse::<u32>().unwrap_err();
    let e: BottleError = raw.clone().into();
    assert_eq!(e, BottleError::BadSteps(raw), "ParseIntError 應該轉成 BadSteps 並保留原本的錯誤");
}

#[test]
fn parse_valid_line() {
    assert_eq!(parse_line("RIGHT 3"), Ok((Dir::Right, 3)), "RIGHT 3 應該讀成往右 3 步");
    assert_eq!(parse_line("  UP   12 "), Ok((Dir::Up, 12)), "多餘的空白不該影響解讀");
}

#[test]
fn empty_line_is_empty() {
    assert_eq!(parse_line("   "), Err(BottleError::Empty), "空白的信應該回報 Empty");
}

#[test]
fn unknown_direction_named() {
    assert_eq!(
        parse_line("NORTH 2"),
        Err(BottleError::UnknownDir("NORTH".to_string())),
        "看不懂的方向要把原字帶回來"
    );
}

#[test]
fn bad_or_missing_steps() {
    assert!(matches!(parse_line("LEFT two"), Err(BottleError::BadSteps(_))), "步數不是數字應該回報 BadSteps");
    assert!(matches!(parse_line("DOWN"), Err(BottleError::BadSteps(_))), "少了步數也應該回報 BadSteps");
}

#[test]
fn decode_whole_letter() {
    assert_eq!(
        decode("RIGHT 2\nDOWN 1"),
        Ok(vec!["MOVE RIGHT".to_string(), "MOVE RIGHT".to_string(), "MOVE DOWN".to_string()]),
        "整封信應該展開成一步一步的指令"
    );
    assert_eq!(
        decode("UP 1\nNORTH 2"),
        Err(BottleError::UnknownDir("NORTH".to_string())),
        "任何一行讀不懂，整封信都要退回"
    );
}
